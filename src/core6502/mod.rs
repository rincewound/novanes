#![allow(dead_code)]

use std::cell::RefCell;


mod opcode;

use super::memory::*;
use opcode::*;

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
    last_opcode: u8,          // Stack pointer
    last_opcode_nmonic: String

}

impl Rico
{
    pub fn new (mut mem: Box<dyn Memory> ) -> Self
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
            last_opcode_nmonic: "<none>".to_string()
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
                    
                    // dispatch opcode
                    let cylces_taken = self.dispatch_opcode(x);
                    
                    self.previouspc = dummypc;
                    self.last_opcode = x;
                    

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
    }

    fn print_cpu_state(&self)
    {
        println!("With:");
        println!("  .X:       {:#2x}", self.x);
        println!("  .Y:       {:#2x}", self.y);
        println!("  .A:       {:#2x}", self.a);
        println!("  .PC:      {:#2x}", self.pc);
        println!("  .S(tack): {:#2x}", self.s);
        println!("  .Stat:    {:#2x}", self.status);
        println!("  .PrevPc:  {:#2x}", self.previouspc);
        println!("  .LastOp:  {}({:#2x})"    , self.last_opcode_nmonic, self.last_opcode);
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

            0x9A => {opcode(rc_self).has_mnemonic("TXS".to_string())
                        .loads_register_u8(RegisterName::X)
                        .to(RegisterName::S)
                        .increments_pc(1)
                        .uses_cycles(2)},  

            x => {                    
                    let e = format!("Encountered bad opcode {:#04x} at {:#06x}", x, pc);
                    //self.print_cpu_state();
                    println!("{}", e);
                    rc_self.borrow().print_cpu_state();
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
        let mut m = RawMemory::new(0x8000);
        m.write_byte(0x0000, opcode);
        let mut r = Rico::new(Box::new(m));
        r.pc = 0x00;
        r.s = 0x00;
        r
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
        cpu.mem.write_byte(0x0001, 0x7E);
        cpu.mem.write_byte(0x0002, 0xCD);
        cpu.mem.write_byte(0x7ECD, 0xAE);
        cpu.execute(1);
        assert_eq!(cpu.a, 0xAE);        
    }

    #[test]
    fn adc_ind_indexed_x_works_as_intended()
    {
        let mut cpu = setup(0x7D);
        cpu.mem.write_byte(0x0001, 0x7E);
        cpu.mem.write_byte(0x0002, 0xCD);
        cpu.mem.write_byte(0x7ECD + 0x20, 0xAE);
        cpu.x = 0x20;
        cpu.execute(1);
        assert_eq!(cpu.a, 0xAE);           
    }

    #[test]
    fn adc_ind_indexed_y_works_as_intended()
    {
        let mut cpu = setup(0x79);
        cpu.mem.write_byte(0x0001, 0x7E);
        cpu.mem.write_byte(0x0002, 0xCD);
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
        cpu.mem.write_byte(0x000A, 0x0F);   // adr hi
        cpu.mem.write_byte(0x000B, 0xAB);   // adr lo
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
        cpu.mem.write_byte(0x0001, 0x09);
        cpu.mem.write_byte(0x0002, 0x10);   
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
}
