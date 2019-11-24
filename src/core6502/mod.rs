#![allow(dead_code)]

use std::cell::RefCell;


mod opcode;

use super::memory::*;
use crate::log;
use opcode::*;

use std::sync::{Arc,Mutex};

pub const CARRY_MASK: u8 = 0x01;
pub const ZERO_MASK: u8 = 0x02;
pub const IRQ_DISABLE_MASK: u8 = 0x04;
pub const DEC_MODE: u8 = 0x08;
pub const OVERFLOW_MASK: u8 = 0x20;
pub const NEG_MASK: u8 = 0x40;

struct OpCodeImpl
{
    name: String,
    implementation: fn() -> u16
}


pub struct Rico
{
    mem: Box<dyn Memory>,

    a: u8,          // Accumulator
    x: u8,
    y: u8,          
    pc: u16,        // Program Counter
    s: u8,          // Stack pointer
    status: u8,      // Also known as P
    previouspc: u16,        // Program Counter
    last_opcode: u8,         
    last_opcode_nmonic: String,
    current_opcode_nmonic: String,
    current_opcode: u8,
    logger: Arc<Mutex<log::logger>>
}

impl Rico
{
    pub fn log(&self, message: String)
    {
        let mut lg = self.logger.lock().unwrap();
        lg.write(message);
    }

    pub fn new (mut mem: Box<dyn Memory>, log: Arc<Mutex<log::logger>> ) -> Self
    {
        for i in 0x4000..0x400F
        {
            mem.write_byte(i, 0x00);
        }

        for i in 0x4010..0x4013
        {
           mem.write_byte(i, 0x00); 
        }

        Rico
        {
            mem: mem,
            a: 0,
            x: 0,
            y: 0,
            pc: 0x8000,     // Check docs, this is the start of the cartridge.
            s: 0xFD,
            status: 0x34, // IRQ disabled,
            previouspc: 0x00,
            last_opcode: 0x00,
            last_opcode_nmonic: "<none>".to_string(),
            current_opcode: 0x00,
            current_opcode_nmonic: "<none>".to_string(),
            logger: log
        }
    }

    pub fn get_memory(&self) -> &Box<dyn Memory>
    {
        &self.mem
    }

    pub fn execute(&mut self, num_cycles: u32)
    {
        let mut cycle_count = 0;
        while cycle_count < num_cycles
        {
            // read opcode
            let opcode = self.mem.read_byte(self.pc as usize);
            match opcode
            {
                Ok(x) => {
                    
                    let dummypc = self.pc;

                    let breakAdr =  0x90e6;
                    if self.pc == breakAdr
                    {
                        self.log("Breakpoint hit.".to_string());
                    }

                    self.current_opcode = x;
                    
                    // dispatch opcode
                    let cylces_taken = self.dispatch_opcode(x);
                    
                    self.previouspc = dummypc;
                    self.last_opcode = x;
                    self.last_opcode_nmonic = self.current_opcode_nmonic.clone();
                    self.current_opcode_nmonic = "<unknown>".to_string();
                    

                    // increase cyclecount -> dispatch tells us how
                    // many cylces it needed. Note that dispatch opcode
                    // *must* modify PC itself, after the opcode
                    // has been dispatched.
                    cycle_count += cylces_taken as u32;
                },
                Err(_) => {
                     self.print_cpu_state();
                     panic!("Bad memory location read."); 
                     }
            }                    
        }

        self.mem.tick(num_cycles);    
    }

    pub fn print_cpu_state(&self)
    {
        self.log(format!("With:"));
        self.log(format!("  .X:                  {:#2x}", self.x));
        self.log(format!("  .Y:                  {:#2x}", self.y));
        self.log(format!("  .A:                  {:#2x}", self.a));
        self.log(format!("  .PC:                 {:#2x}", self.pc));
        self.log(format!("  .S(tack):            {:#2x}", self.s));
        self.log(format!("  .Stat:               {:#2x}", self.status));
        self.log(format!("  .Cur Op:             {}({:#2x}) @ {:#2x}"     , self.current_opcode_nmonic, self.current_opcode, self.pc));
        self.log(format!("  .Last Successful op: {}({:#2x}) @ {:#2x}"     , self.last_opcode_nmonic, self.last_opcode, self.previouspc));   
    }

    fn dispatch_opcode(&mut self, oc: u8) -> u16
    {
        let pc = self.pc;
        let rc_self = RefCell::new(self);    
        match oc
        {
            0x00 => { opcode(rc_self).has_mnemonic("NOP".to_string())
                                     .increments_pc(1)
                                     .uses_cycles(1) },
            
            0x09 => { opcode(rc_self).has_mnemonic("ORA #$nn".to_string())
                                     .loads_immediate()
                                     .or_with_accumulator()
                                     .increments_pc(2)
                                     .uses_cycles(2) },

            0x10 => { opcode(rc_self).has_mnemonic("BPL".to_string())
                                     .loads_immediate()
                                     .jumps_relative_if_statusbit(NEG_MASK, false)
                                     .increments_pc(2)
                                     .uses_cycles(2) },

             0x20 => { opcode(rc_self).has_mnemonic("JSR $hhll".to_string())
                                     .loads_immediate_16bit()
                                     .jumps_to_subroutine()
                                     .increments_pc(0)      // Is done internally ?!?
                                     .uses_cycles(6) },
            
            0x2C => { opcode(rc_self).has_mnemonic("BIT".to_string())
                            .loads_immediate_16bit()
                            .performs_bit_test()
                            .increments_pc(3)
                            .uses_cycles(4) },
            
            0x29 => { opcode(rc_self).has_mnemonic("AND#".to_string())
                .loads_immediate()
                .and_with_accumulator()
                .increments_pc(2)
                .uses_cycles(2) },
            
            0x4C => { opcode(rc_self).has_mnemonic("JMP".to_string())
                .loads_immediate_16bit()
                .jumps_to_address()
                .increments_pc(0)
                .uses_cycles(3) },
            
            0xEE => { opcode(rc_self).has_mnemonic("INC $HHLL".to_string())
                .loads_immediate_16bit()
                .increments_address(1)
                .increments_pc(3)
                .uses_cycles(6) },
            
            0x60 => { opcode(rc_self).has_mnemonic("RTS".to_string())
                                     .returns_from_subroutine()
                                     .increments_pc(0)      // Is done internally ?!?
                                     .uses_cycles(6) },
            
            0xB0 => { opcode(rc_self).has_mnemonic("BCS".to_string())
                                     .loads_immediate_16bit()
                                     .jumps_relative_if_statusbit(CARRY_MASK, true)
                                     .increments_pc(2)
                                     .uses_cycles(2) },
            
            0xD0 => { opcode(rc_self).has_mnemonic("BNE".to_string())
                            .loads_immediate_16bit()
                            .jumps_relative_if_statusbit(ZERO_MASK, false)
                            .increments_pc(2)
                            .uses_cycles(2) },

            0x78 => {opcode(rc_self).has_mnemonic("SEI".to_string())
                                    .toggles_cpu_bit(IRQ_DISABLE_MASK, true)
                                    .increments_pc(1)
                                    .uses_cycles(2)},

            // ADC ------------------------------------------------------            
            0x69 => { opcode(rc_self).has_mnemonic("ADC#".to_string())
                                    .loads_immediate()
                                    .adds_to_accumulator()
                                    .increments_pc(2)
                                    .uses_cycles(2) },

            0x6D => { opcode(rc_self).has_mnemonic("ADC$hhll".to_string())
                                    .loads_indirect(0)
                                    .adds_to_accumulator()
                                    .increments_pc(3)
                                    .uses_cycles(4) },

            0x7D => { opcode(rc_self).has_mnemonic("ADC$hhll, X".to_string())
                                    .loads_indirect_indexed_x()
                                    .adds_to_accumulator()
                                    .increments_pc(3)
                                    .uses_cycles(4) },

            0x79 => { opcode(rc_self).has_mnemonic("ADC$hhll, Y".to_string())
                                    .loads_indirect_indexed_y()
                                    .adds_to_accumulator()
                                    .increments_pc(3)
                                    .uses_cycles(4) },

            0x65 => { opcode(rc_self).has_mnemonic("ADC$ll".to_string())
                                    .loads_from_zeropage(0)
                                    .adds_to_accumulator()
                                    .increments_pc(2)
                                    .uses_cycles(3) },

            0x75 => { opcode(rc_self).has_mnemonic("ADC$ll,X".to_string())
                                    .loads_from_zeropage_indexed_x()
                                    .adds_to_accumulator()
                                    .increments_pc(2)
                                    .uses_cycles(4) },

            0x61 => { opcode(rc_self).has_mnemonic("ADC($ll,X)".to_string())
                                    .loads_from_zeropage_indirect_indexed_x()
                                    .adds_to_accumulator()
                                    .increments_pc(2)
                                    .uses_cycles(6) },

            
            0x71 => { opcode(rc_self).has_mnemonic("ADC($ll),X".to_string())
                                    .loads_from_zeropage_indirect_postindexed_y()
                                    .adds_to_accumulator()
                                    .increments_pc(2)
                                    .uses_cycles(5) },
            
            // SBC ----------------------------------------------------------
            0xE9 => { opcode(rc_self).has_mnemonic("SBC #$nn".to_string())
                                    .loads_immediate()
                                    .subtracts_from_accumulator()
                                    .increments_pc(2)
                                    .uses_cycles(2)},

            0xA9 => { opcode(rc_self).has_mnemonic("LDA #$nn".to_string())
                                    .loads_immediate()
                                    .to(RegisterName::A)
                                    .increments_pc(2)
                                    .uses_cycles(2)},

            0xAD => { opcode(rc_self).has_mnemonic("LDA $hhll".to_string())
                                    .loads_indirect(0)
                                    .to(RegisterName::A)
                                    .increments_pc(3)
                                    .uses_cycles(4)},


            // Transfer instructions:
            0xAA => {opcode(rc_self).has_mnemonic("TAX".to_string())
                                    .loads_register_u8(RegisterName::A)
                                    .to(RegisterName::X)
                                    .increments_pc(1)
                                    .uses_cycles(2)},

            0xA8 => {opcode(rc_self).has_mnemonic("TAY".to_string())
                                    .loads_register_u8(RegisterName::A)
                                    .to(RegisterName::Y)
                                    .increments_pc(1)
                                    .uses_cycles(2)},

            0x8A => {opcode(rc_self).has_mnemonic("TXA".to_string())
                        .loads_register_u8(RegisterName::X)
                        .to(RegisterName::A)
                        .increments_pc(1)
                        .uses_cycles(2)}, 

            0x98 => {opcode(rc_self).has_mnemonic("TYA".to_string())
                        .loads_register_u8(RegisterName::Y)
                        .to(RegisterName::A)
                        .increments_pc(1)
                        .uses_cycles(2)},         


            0xBA => {opcode(rc_self).has_mnemonic("TSX".to_string())
                        .loads_register_u8(RegisterName::S)
                        .to(RegisterName::X)
                        .increments_pc(1)
                        .uses_cycles(2)},  

            0xBD => {opcode(rc_self).has_mnemonic("LDA $hhll, x".to_string())
                        .loads_indirect_indexed_x()
                        .to(RegisterName::X)
                        .increments_pc(3)
                        .uses_cycles(4)
                    },    

            0x9A => {opcode(rc_self).has_mnemonic("TXS".to_string())
                        .loads_register_u8(RegisterName::X)
                        .to(RegisterName::S)
                        .increments_pc(1)
                        .uses_cycles(2)},  
            
            0xD8 => {opcode(rc_self).has_mnemonic("CLD".to_string())
                        .toggles_cpu_bit(DEC_MODE, false)
                        .increments_pc(1)
                        .uses_cycles(2)},  

            0x8D => {opcode(rc_self).has_mnemonic("STA $hhll".to_string())
                        .stores(RegisterName::A)
                        .to_immediate_address()
                        .increments_pc(3)
                        .uses_cycles(4)}

            0x85 => {opcode(rc_self).has_mnemonic("STA $ll".to_string())
                        .stores(RegisterName::A)
                        .to_zeropage()
                        .increments_pc(2)
                        .uses_cycles(3)}
            
            0x91 => {opcode(rc_self).has_mnemonic("STA ($ll), Y".to_string())
                        .stores(RegisterName::A)
                        .to_indirect_address(RegisterName::Y)
                        .increments_pc(2)
                        .uses_cycles(6)}

            0x99 => {opcode(rc_self).has_mnemonic("STA $hhll, Y".to_string())
                        .stores(RegisterName::A)
                        .to_immediate_address_with_register_offset(RegisterName::Y)
                        .increments_pc(3)
                        .uses_cycles(5)}
            
            0x86 => {opcode(rc_self).has_mnemonic("STX $ll".to_string())
                        .stores(RegisterName::X)
                        .to_zeropage()
                        .increments_pc(2)
                        .uses_cycles(3)}

            0xA2 => {opcode(rc_self).has_mnemonic("LDX #$nn".to_string())
                        .loads_immediate()
                        .to(RegisterName::X)
                        .increments_pc(2)
                        .uses_cycles(2)}
            
            0xA0 => {opcode(rc_self).has_mnemonic("LDY #$nn".to_string())
                        .loads_immediate()
                        .to(RegisterName::Y)
                        .increments_pc(2)
                        .uses_cycles(2)}

            0xC9 => {opcode(rc_self).has_mnemonic("CMP #$nn".to_string())
                        .loads_immediate()
                        .compares_value(RegisterName::A)
                        .increments_pc(2)
                        .uses_cycles(2)}     

            0xCA => {opcode(rc_self).has_mnemonic("DEX".to_string())
                        .decrements_register(RegisterName::X)
                        .increments_pc(1)
                        .uses_cycles(2)}     

            0x88 => {opcode(rc_self).has_mnemonic("DEY".to_string())
                        .decrements_register(RegisterName::Y)
                        .increments_pc(1)
                        .uses_cycles(2)}
            
            0xC8 => {opcode(rc_self).has_mnemonic("INY".to_string())
                        .increments_register(RegisterName::Y)
                        .increments_pc(1)
                        .uses_cycles(2)}

            0xE0 => { opcode(rc_self).has_mnemonic("CPX #$nn".to_string())
                        .loads_immediate()
                        .compares_value(RegisterName::X)
                        .increments_pc(2)
                        .uses_cycles(2)},

            0xC0 => { opcode(rc_self).has_mnemonic("CPY #$nn".to_string())
                        .loads_immediate()
                        .compares_value(RegisterName::Y)
                        .increments_pc(2)
                        .uses_cycles(2)},                                   

            x => {                    
                    let e = format!("Encountered bad opcode {:#04x} at {:#06x}", x, pc);
                    //self.print_cpu_state();
                    let s = rc_self.borrow();
                    s.log(format!("{}", e));
                    s.print_cpu_state();
                    s.logger.lock().unwrap().to_console();
                    panic!(e)
                }
        }
    }

}



#[cfg(test)]
mod opcodetests 
{
    use std::panic;
    //use super::memory::*;
    use crate::core6502::*;
    
    fn setup(opcode: u8) -> crate::core6502::Rico
    {
        let logger = Arc::new(Mutex::new(log::logger::new()));
        let mut m = RawMemory::new(0x8000);
        m.write_byte(0x0000, opcode);
        let mut r = Rico::new(Box::new(m), logger);
        r.pc = 0x00;
        r.s = 0x00;
        r
    }

    fn hasValueAt(cpu: &mut crate::core6502::Rico, adr: u16, val: u8) -> bool
    {
        let v = cpu.mem.read_byte(adr as usize).unwrap();
        return v == val;
    }

    fn teardown()
    {

    }

    #[test]
    fn nop_works_as_intended() 
    {
        let mut cpu = setup(0x00);
        cpu.execute(1);
        assert_eq!(cpu.pc, 0x0001)
    }

    #[test]
    fn adc_immediate_works_as_intended()
    {
        let mut cpu = setup(0x69);
        cpu.mem.write_byte(0x0001, 44);
        cpu.a = 10;
        cpu.execute(1);
        assert_eq!(cpu.pc, 0x0002);
        assert_eq!(cpu.a, 54);
        assert_eq!(cpu.status & ZERO_MASK, 0);
    }

    #[test]
    fn adc_sets_zero_flag_if_zero()
    {
        let mut cpu = setup(0x69);
        cpu.mem.write_byte(0x0001, 0);
        cpu.a = 0;
        cpu.execute(1);
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.status & ZERO_MASK, ZERO_MASK);
    }

    #[test]
    fn adc_sets_carry_flag_if_overflow()
    {        
        let mut cpu = setup(0x69);
        cpu.mem.write_byte(0x0001, 0xFF);
        cpu.a = 2;
        cpu.execute(1);
        assert_eq!(cpu.a, 1);
        assert_eq!(cpu.status & CARRY_MASK, CARRY_MASK);
    }

    #[test]
    fn adc_honors_carry_flag()
    {
        let mut cpu = setup(0x69);        
        cpu.mem.write_byte(0x0001, 0x1);
        cpu.status = cpu.status | CARRY_MASK;
        cpu.a = 1;
        cpu.execute(1);
        assert_eq!(cpu.a, 3);
        assert_eq!(cpu.status & CARRY_MASK, 0);
    }

    #[test]
    fn adc_ind_works_as_intended()
    {
        let mut cpu = setup(0x6D);
        cpu.mem.write_byte(0x0001, 0xCD);
        cpu.mem.write_byte(0x0002, 0x7E);
        cpu.mem.write_byte(0x7ECD, 0xAE);
        cpu.execute(1);
        assert_eq!(cpu.a, 0xAE);        
    }

    #[test]
    fn adc_ind_indexed_x_works_as_intended()
    {
        let mut cpu = setup(0x7D);
        cpu.mem.write_byte(0x0001, 0xCD);
        cpu.mem.write_byte(0x0002, 0x7E);
        cpu.mem.write_byte(0x7ECD + 0x20, 0xAE);
        cpu.x = 0x20;
        cpu.execute(1);
        assert_eq!(cpu.a, 0xAE);           
    }

    #[test]
    fn adc_ind_indexed_y_works_as_intended()
    {
        let mut cpu = setup(0x79);
        cpu.mem.write_byte(0x0001, 0xCD);
        cpu.mem.write_byte(0x0002, 0x7E);
        cpu.mem.write_byte(0x7ECD + 0x40, 0xAE);
        cpu.y = 0x40;
        cpu.execute(1);
        assert_eq!(cpu.a, 0xAE);           
    }

    #[test]
    fn adc_zeropage_indexed_x_works_as_intended()
    {
        let mut cpu = setup(0x75);
        cpu.mem.write_byte(0x0001, 0x7E); // offset at which to find the operand       
        cpu.mem.write_byte(0x007E, 0x44); // actual operand
        cpu.a = 0x20;
        cpu.execute(1);
        assert_eq!(cpu.a, 0x20 + 0x44);           
    }

    #[test]
    fn adc_zeropage_indexed_works_as_intended()
    {
        let mut cpu = setup(0x65);
        cpu.mem.write_byte(0x0001, 0x0F);
        cpu.mem.write_byte(0x000F, 0x7E);        
        cpu.x = 0x10;
        cpu.a = 0x20;
        cpu.execute(1);
        assert_eq!(cpu.a, 0x20 + 0x7E);           
    }

    #[test]
    fn adc_indirect_x_indexed_works_as_intended()
    {
        let mut cpu = setup(0x61);
        cpu.mem.write_byte(0x0001, 0x09);
        cpu.mem.write_byte(0x000A, 0xAB);   // adr hi
        cpu.mem.write_byte(0x000B, 0x0F);   // adr lo
        cpu.mem.write_byte(0x0FAB, 0x20);   // adr lo
        cpu.x = 0x01;
        cpu.a = 0x20;
        cpu.execute(1);
        assert_eq!(cpu.a, 0x40);           
    }

    #[test]
    fn adc_indirect_y_postindexed_works_as_intended()
    {
        let mut cpu = setup(0x71);
        cpu.mem.write_byte(0x0001, 0x10);
        cpu.mem.write_byte(0x0002, 0x09);   
        cpu.mem.write_byte(0x0930, 0xAB);
        cpu.y = 0x20;
        cpu.a = 0x10;
        cpu.execute(1);
        assert_eq!(cpu.a, 0x10 + 0xAB);           
    }

    // Note: The different methods of each op getting to its data has been mostly
    // testsd with ADC, so we only test the core behavior of the sbc opcode here
    #[test]
    fn sbc_works_as_intended()
    {
       let mut cpu = setup(0xE9); 
       cpu.mem.write_byte(0x0001, 22);
       cpu.a = 27;
       cpu.execute(1);
       assert_eq!(cpu.a, 5);
       assert_eq!(cpu.status & CARRY_MASK, 0x00);
    }

    #[test]
    fn sbc_sets_carry_if_underflow()
    {
        let mut cpu = setup(0xE9); 
        cpu.mem.write_byte(0x0001, 27);
        cpu.a = 22;
        cpu.execute(1);
        assert_eq!(cpu.a, 5);
        assert_eq!(cpu.status & CARRY_MASK, CARRY_MASK);
    }

    #[test]
    fn tax_works_as_intended()
    {
        let mut cpu = setup(0xAA); 
        cpu.a = 22;
        cpu.execute(1);
        assert_eq!(cpu.x, 22);
    }

    #[test]
    fn tay_works_as_intended()
    {
        let mut cpu = setup(0xA8); 
        cpu.a = 27;
        cpu.execute(1);
        assert_eq!(cpu.y, 27);
    }

    #[test]
    fn txa_works_as_intended()
    {
         let mut cpu = setup(0x8A); 
        cpu.x = 45;
        cpu.execute(1);
        assert_eq!(cpu.a, 45);
    }

    #[test]
    fn tya_works_as_intended()
    {
        let mut cpu = setup(0x98); 
        cpu.y = 7;
        cpu.execute(1);
        assert_eq!(cpu.a, 7);
    }

    #[test]
    fn tsx_works_as_intended()
    {
        let mut cpu = setup(0xBA); 
        cpu.s = 37;
        cpu.execute(1);
        assert_eq!(cpu.x, 37);
    }

    #[test]
    fn txs_works_as_intended()
    {
        let mut cpu = setup(0x9A); 
        cpu.x = 51;
        cpu.execute(1);
        assert_eq!(cpu.s, 51);
    }

    #[test]
    fn sei_sets_irq_disble_flag()
    {
        let mut cpu = setup(0x78); 
        cpu.execute(1);
        assert_eq!(cpu.status & IRQ_DISABLE_MASK, IRQ_DISABLE_MASK);
    }

    #[test]
    fn cld_clears_decimal_flag()
    {
        let mut cpu = setup(0xd8); 
        cpu.execute(1);
        assert_eq!(cpu.status & DEC_MODE, 0x00);        
    }

    #[test]
    fn lda_loads_accumulator()
    {
        let mut cpu = setup(0xa9);
        cpu.mem.write_byte(0x0001, 0x10);
        cpu.execute(1);
        assert_eq!(cpu.a, 0x10);
    }

    #[test]
    fn ldx_loads_x_reg()
    {
        let mut cpu = setup(0xa2);
        cpu.mem.write_byte(0x0001, 0x10);
        cpu.execute(1);
        assert_eq!(cpu.x, 0x10);       
    }

    #[test]
    fn ldy_loads_y_reg()
    {
        let mut cpu = setup(0xa0);
        cpu.mem.write_byte(0x0001, 0x10);
        cpu.execute(1);
        assert_eq!(cpu.y, 0x10);       
    }

    #[test]
    fn ldx_indexed_x_loads_x()
    {
        let mut cpu = setup(0xbd);
        cpu.mem.write_byte(0x0001, 0x11);
        cpu.mem.write_byte(0x0002, 0x12);
        cpu.x = 0x10;
        cpu.mem.write_byte(0x1221, 0xAB);
        cpu.execute(1);
        assert_eq!(cpu.x, 0xAB);       
    }

    #[test]
    fn dex_decrements_x()
    {
        let mut cpu = setup(0xca);
        cpu.x = 47;
        cpu.execute(1);
        assert_eq!(cpu.x, 46);       
    }

    #[test]
    fn cmp_sets_carry_if_comparand_is_smaller()
    {
        let mut cpu = setup(0xc9);
        cpu.mem.write_byte(0x0001, 0x11);       
        cpu.a = 0x21;
        cpu.execute(1);
        assert_eq!(cpu.status & CARRY_MASK, CARRY_MASK);
    }

    #[test]
    fn cmp_sets_zero_if_comparand_is_equal()
    {
        let mut cpu = setup(0xc9);
        cpu.mem.write_byte(0x0001, 0x11);       
        cpu.a = 0x11;
        cpu.execute(1);
        assert_eq!(cpu.status & ZERO_MASK, ZERO_MASK);
    }

    #[test]
    fn cmp_sets_neg_if_comparand_is_larger()
    {
        let mut cpu = setup(0xc9);
        cpu.mem.write_byte(0x0001, 0x21);       
        cpu.a = 0x11;
        cpu.execute(1);
        assert_eq!(cpu.status & NEG_MASK, NEG_MASK);
    }

    #[test]
    fn stx_works()
    {
        let mut cpu = setup(0x86);
        cpu.mem.write_byte(0x0001, 0x24);
        cpu.x = 0xFA;
        cpu.execute(1);
        assert_eq!(true, hasValueAt(&mut cpu, 0x24, 0xFA))
    }

}
