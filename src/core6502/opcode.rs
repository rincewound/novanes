
use crate::core6502::*;

use std::cell::RefCell;
use std::fmt::{Display, Formatter, Result};

pub struct Opcode<'a>
{
    cpu: RefCell <&'a mut crate::core6502::Rico>
}

#[derive(Debug)]
pub enum RegisterName
{
    A,
    X,
    Y,
    PC,
    S,
    Status
}

impl Display for RegisterName
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub struct LoadResult<'a>
{
    val: u16,
    origin: Opcode<'a>
}


impl<'a> LoadResult<'a>
{ 
    pub fn log(&self, message: String)
    {
        let cpu = self.origin.cpu.borrow();
        cpu.log(message);
    }

    pub fn new16(value: u16, source: Opcode<'a>) -> Self
    {
        LoadResult{
            val : value,
            origin : source
            }
    }

    pub fn new8(value: u8, source: Opcode<'a>) -> Self
    {
        LoadResult{
            val : value as u16,
            origin : source
            }
    }

    pub fn to(self, target: RegisterName) -> Opcode<'a>
    {
        // ToDo: this method might need to adjust status registers!
        self.toggle_cpu_bit(NEG_MASK, (self.val as u8 & 0x80) != 0);

        match target{
            RegisterName::A => self.origin.cpu.borrow_mut().a = self.val as u8,
            RegisterName::X => self.origin.cpu.borrow_mut().x = self.val as u8,
            RegisterName::Y => self.origin.cpu.borrow_mut().y = self.val as u8,
            RegisterName::PC => self.origin.cpu.borrow_mut().pc = self.val,
            RegisterName::S => self.origin.cpu.borrow_mut().s = self.val as u8,
            RegisterName::Status => self.origin.cpu.borrow_mut().status = self.val as u8,
        }
        self.log(format!("          V({:#2x}) -> {}", self.val, target));
        self.origin
    }

    fn toggle_cpu_bit(&self, bit: u8, newval: bool)
    {
        let mut cpu = self.origin.cpu.borrow_mut();
        if newval
        {
            cpu.status |= bit;
        }
        else
        {
            cpu.status = cpu.status & !bit;
        }
    }

    pub fn adds_to_accumulator(self) -> Opcode<'a>
    {
        let mut tmpval : u16 = 0;
        {
            let mut cpu = self.origin.cpu.borrow_mut();
            if cpu.status & CARRY_MASK == CARRY_MASK
            {
                tmpval += 1;
            }

            tmpval += cpu.a as u16 + self.val;
            let aval = tmpval & 0xFF;
            cpu.a = aval as u8;                 
        }

        self.toggle_cpu_bit(ZERO_MASK, (tmpval & 0xFF) == 0);
        self.toggle_cpu_bit(CARRY_MASK, tmpval > 255);   

        self.origin
    }

    pub fn jumps_relative_if_statusbit(self, statusbit: u8, val: bool) -> Opcode<'a>
    {
        let mut logstring = String::from("foo");
        let mut isBitSet: bool = false;
        {
            let mut cpu = self.origin.cpu.borrow_mut();
            let actualVal = self.val as i8;
            isBitSet = (cpu.status & statusbit) != 0;

            if isBitSet == val
            {                
                let next_pc = (cpu.pc as i32 + actualVal as i32) as u16;                
                cpu.pc = next_pc;                
                logstring = format!("          {:#4x} + {} = #({:#4x}) -> PC", cpu.pc, actualVal, next_pc);                              
            }
        }

        if isBitSet
        {
            self.log(logstring);
        }

        self.origin
    }

    pub fn jumps_to_subroutine(self) -> Opcode<'a>
    {
        {
            //push pc to stack
            let mut cpu = self.origin.cpu.borrow_mut();
            let nextpc = cpu.pc + 2;
            cpu.pc = self.val;
            let write0 = cpu.s as usize;
            let write1 = (cpu.s - 1) as usize;
            cpu.mem.write_byte(write0, (nextpc & 0xFF) as u8);
            cpu.mem.write_byte(write1, ((nextpc & 0xFF00) >> 8) as u8);
            cpu.s -= 2;
        }
        
        self.origin
    }

    pub fn subtracts_from_accumulator(self) -> Opcode<'a>
    {
        let mut tmpval : i16 = 0;
        let mut set_carry = false;

        {
            let mut cpu = self.origin.cpu.borrow_mut();

            tmpval += cpu.a as i16 - self.val as i16;

            if cpu.status & CARRY_MASK == CARRY_MASK
            {
                tmpval |= 0x80;
            }
            
            if tmpval < 0
            {
                tmpval = 256 - tmpval;
                set_carry = true;
                // todo: Set overflow flag here!
            }

            let aval = tmpval as u16 & 0xFF;
            cpu.a = aval as u8;                 
        }

        self.toggle_cpu_bit(ZERO_MASK, (tmpval & 0xFF) == 0);
        self.toggle_cpu_bit(CARRY_MASK, set_carry);   

        self.origin
    }

    pub fn xor_with_accumulator(self) -> Opcode<'a>
    {
        let result: u8;
        {
            let mut cpu = self.origin.cpu.borrow_mut();
            result = cpu.a ^ self.val as u8;
            cpu.a = result;            
        }
        self.toggle_cpu_bit(ZERO_MASK, result == 0);
        self.origin
    }

    pub fn or_with_accumulator(self) -> Opcode<'a>
    {
        let result: u8;
        {
            let mut cpu = self.origin.cpu.borrow_mut();
            result = cpu.a | self.val as u8;
            cpu.a = result;            
        }
        self.toggle_cpu_bit(ZERO_MASK, result == 0);
        self.origin
    }

    pub fn and_with_accumulator(self) -> Opcode<'a>
    {
        let result: u8;
        {
            let mut cpu = self.origin.cpu.borrow_mut();
            result = cpu.a & self.val as u8;
            cpu.a = result;            
        }
        self.toggle_cpu_bit(ZERO_MASK, result == 0);
        self.origin
    }

    pub fn compares_value(self) -> Opcode<'a>
    {
        let res : i16;
        {
            let cpu = self.origin.cpu.borrow();
            res = cpu.a as i16 - self.val as i16;
        }
        
        self.toggle_cpu_bit(NEG_MASK, false);
        self.toggle_cpu_bit(ZERO_MASK, false);
        self.toggle_cpu_bit(CARRY_MASK, false);

        if res < 0
        {
            self.toggle_cpu_bit(NEG_MASK, true);
        }

        if res == 0
        {
            self.toggle_cpu_bit(ZERO_MASK, true);
        }

        if res > 0
        {
            self.toggle_cpu_bit(CARRY_MASK, true);
        }
        

        self.origin
    }
}

pub struct StoreCommand<'a>
{
    val: u16,
    origin: Opcode<'a>
}

impl<'a> StoreCommand<'a>
{     
    pub fn log(&self, message: String)
    {
        let cpu = self.origin.cpu.borrow();
        cpu.log(message);
    }

    pub fn new16(value: u16, source: Opcode<'a>) -> Self
    {
        StoreCommand
            {
                val : value,
                origin : source
            }
    }

    pub fn new8(value: u8, source: Opcode<'a>) -> Self
    {
        StoreCommand
            {
                val : value as u16,
                origin : source
            }
    }

    pub fn to_immediate_address(self) -> Opcode<'a>
    {        
        let logstring: String;
        {        
            let mut cpu = self.origin.cpu.borrow_mut();
            let readAdr = (cpu.pc + 1) as usize;
            let adr = cpu.mem.read_u16(readAdr).unwrap() as usize;
            logstring = format!("       #({}) -> #({})", self.val as u8, adr);
            cpu.mem.write_byte(adr, self.val as u8);
        }
        self.log(logstring);
        self.origin
    }

    pub fn to_immediate_address_with_offset(self) -> Opcode<'a>
    {
        self.origin
    }

    pub fn to_zeropage(self) -> Opcode<'a>
    { 
        let logstring: String;
        {
            let mut cpu = self.origin.cpu.borrow_mut();
            let readAdr = (cpu.pc + 1) as usize;
            let adr = cpu.mem.read_byte(readAdr).unwrap() as usize;
            logstring = format!("          #({}) -> {:#4x}", self.val as u8, adr);
            cpu.mem.write_byte(adr, self.val as u8);
        }
        self.log(logstring);
        self.origin
    }

    pub fn to_zeropage_with_offset(self) -> Opcode<'a>
    {
        self.origin
    }
}


impl<'a> Opcode<'a>
{
    pub fn log(&self, message: String)
    {
        let cpu = self.cpu.borrow();
        cpu.log(message);
    }

    pub fn new(cpu:  RefCell <&'a mut crate::core6502::Rico>) -> Self
    {
        Opcode{cpu: cpu}
    }

    pub fn uses_cycles(&self, num_cycles: u16) -> u16
    {
        num_cycles
    }

    pub fn increments_pc(self, num_bytes: u16) -> Opcode<'a>
    {
        self.cpu.borrow_mut().pc += num_bytes;
        self
    }

    pub fn decrements_register(self, reg: RegisterName) -> Opcode<'a>
    {
        {
            let mut cpu = self.cpu.borrow_mut();
            let mut val: u8;

            match reg
            {
                RegisterName::A => val = cpu.a,
                RegisterName::X => val = cpu.x,
                RegisterName::Y => val = cpu.y,
                _ => panic!("Unsupported register")
            };

            if val > 0
            {
                val -= 1;
                if val == 0
                {
                    cpu.status |= ZERO_MASK;
                }                
            }
            else
            {
                val = 0xFF;
                
                cpu.status |= NEG_MASK;
            }

            match reg
            {
                RegisterName::A => cpu.a = val,
                RegisterName::X => cpu.x = val,
                RegisterName::Y => cpu.y = val,
                _ => panic!("Unsupported register")
            };
        }

        self
    }

    pub fn has_mnemonic(self, nmonic: String ) -> Opcode<'a>
    {
        {
            let mut pc: u16 = 0;
            {
            let mut cpu = self.cpu.borrow_mut();
            cpu.current_opcode_nmonic = nmonic.clone();
            pc = cpu.pc;
            }
            self.log(format!("{:#4x}    {}", pc, nmonic));
        }
        self
    }

    fn load_u16(&self, adr: u16) -> u16
    {
        let mut res: u16 = 0;
        {
            let mut cpu = self.cpu.borrow_mut();
            res = cpu.mem.read_u16(adr as usize).unwrap();
        }
        self.log(format!("          #({:#2x}) <- {:#4x}", res, adr));
        res
    }

    fn fetch_u8(&self, adr: u16) -> u8
    {
        let mut cpu = self.cpu.borrow_mut();
        cpu.mem.read_byte(adr as usize).unwrap()
    }

    fn load_u8_from_mem(self, adr: u16) -> LoadResult<'a>
    {
        let result = self.cpu.borrow_mut().mem.read_byte(adr as usize);
        let pc = self.cpu.borrow().pc;

        match result
        {
            Ok(val) => {
                self.log(format!("          #({:#2x}) <- {:#4x}", val, adr));
                LoadResult::new8(val, self)
            },
            Err(_) => 
            {
                self.cpu.borrow().print_cpu_state();
                self.log(format!("access violation at pc {:#4x}", pc + 1));
                panic!("failed to read from {:#4x}", adr);
            }
        }  
    }

    fn read_register(&self, reg: RegisterName) -> u8
    {
        let mut val : u8;
        match reg
        {
            RegisterName::A => val = self.cpu.borrow().a,
            RegisterName::X => val = self.cpu.borrow().x,
            RegisterName::Y => val = self.cpu.borrow().y,
            RegisterName::S => val = self.cpu.borrow().s,
            RegisterName::Status => val = self.cpu.borrow().status,
            _ => panic!("cannot read this register as 8 bit value")
        }

        self.log(format!("          {} -> #({})", reg, val));

        val
    }

    fn read_pc(&self) -> u16
    {
        self.cpu.borrow().pc
    }

    pub fn loads_register_u8(self, reg_name: RegisterName) -> LoadResult<'a>
    {
        let reg_val = self.read_register(reg_name);
        LoadResult::new8(reg_val, self)
    }

    pub fn loads_immediate(self) -> LoadResult<'a>
    {        
        let load_adr: u16;
        {
            load_adr = self.cpu.borrow().pc + 1;
        }
        self.load_u8_from_mem(load_adr)      
    }

    pub fn loads_immediate_16bit(self) -> LoadResult<'a>
    {
        let load_adr: u16;
        {
            load_adr = self.cpu.borrow().pc + 1;
        }
        let loadVal = self.load_u16(load_adr);
        LoadResult::new16(loadVal, self) 
    }

    pub fn loads_indirect(self, offset: u8) -> LoadResult<'a>
    {
        let load_adr: u16;
        {
            load_adr = self.load_u16(self.read_pc() + 1) + offset as u16;            
        }
        self.load_u8_from_mem(load_adr)  
    }

    pub fn loads_indirect_indexed_x(self) -> LoadResult<'a>
    {
        let val = self.read_register(RegisterName::X);
        self.loads_indirect(val)  
    }

    pub fn loads_indirect_indexed_y(self) -> LoadResult<'a>
    {    
        let val = self.read_register(RegisterName::Y);  
        self.loads_indirect(val)
    }

    pub fn loads_from_zeropage(self, offset: u8) -> LoadResult<'a>
    {      
        let adr = self.read_pc() + 1;
        {
            let adrread = self.cpu.borrow_mut().mem.read_byte(adr as usize);
            match adrread
            {
                Ok(val) => self.load_u8_from_mem((val + offset) as u16),
                Err(_) => panic!("bad read from zeropage.")
            }
        }
    }

    pub fn loads_from_zeropage_indexed_x(self) -> LoadResult<'a>
    {
        let xval: u8;
        {
            xval = self.cpu.borrow().x;
        }
        self.loads_from_zeropage(xval)

    }

    pub fn loads_from_zeropage_indexed_y(self) -> LoadResult<'a>
    {
        let yval: u8;
        {
            yval = self.cpu.borrow().y;
        }
        self.loads_from_zeropage(yval)
    }

    pub fn loads_from_zeropage_indirect_indexed_x(self) -> LoadResult<'a>
    {
        let xval = self.read_register(RegisterName::X);      
        let pc = self.read_pc();             
        let load_adr_base = self.fetch_u8(pc + 1) + xval;
        let effective_adr = self.load_u16(load_adr_base as u16);
        self.load_u8_from_mem(effective_adr)
    }

    pub fn loads_from_zeropage_indirect_postindexed_y(self) -> LoadResult<'a>
    {
        let yval = self.read_register(RegisterName::Y) as u16;      
        let pc = self.read_pc();             
        let load_adr_base = self.load_u16(pc + 1) + yval;
        self.load_u8_from_mem(load_adr_base)
    }

    pub fn stores(self, source: RegisterName) -> StoreCommand<'a>
    {
        let val = self.read_register(source);
        StoreCommand::new8(val, self)
    }

    pub fn toggles_cpu_bit(self, bit: u8, newval: bool)-> Opcode<'a>
    {
        {
            let mut cpu = self.cpu.borrow_mut();
            if newval
            {
                cpu.status |= bit;
            }
            else
            {
                cpu.status = cpu.status & !bit;
            }
        }
        self
    }
}

pub fn opcode(cpu: RefCell <&mut crate::core6502::Rico>) -> Opcode
{
    Opcode::new(cpu)
}

#[cfg(test)]
mod store_command_tests
{
    use crate::core6502::*;
    use crate::core6502::opcode::*;
    use std::panic;  

    pub fn run_test<T>(val: u8, test: T) -> () 
        where T: FnOnce(StoreCommand) -> () + panic::UnwindSafe
    {
        let result = panic::catch_unwind(|| {
            let logger = Arc::new(Mutex::new(log::logger::new()));
            let mem = RawMemory::new(0x10000);
            let mut cpu = Rico::new(Box::new(mem), logger);
            let oc = opcode(RefCell::new(&mut cpu));
            let sc = StoreCommand::new8(val, oc);
            test(sc)
        });
        assert!(result.is_ok())
    } 

    fn CheckHasValAt(sr: &Opcode, adr: usize, val: u8)
    {
        let result = sr.cpu.borrow_mut().mem.read_byte(0x1020);

        match result
        {
            Ok(v) => assert_eq!(v, val),
            Err(e) => assert_eq!(true, false)
        }
    }

    #[test]
    pub fn store_to_immediate_adress_works()
    {
        run_test(0xAB, |sr|
        {
            sr.origin.cpu.borrow_mut().mem.write_byte(0x8001, 0x20);
            sr.origin.cpu.borrow_mut().mem.write_byte(0x8002, 0x10);
            let oc = sr.to_immediate_address();
            CheckHasValAt(&oc, 0x1020, 0xAB)            
        })
    }


}


#[cfg(test)]
mod load_result_tests 
{
    use crate::core6502::*;
    use crate::core6502::opcode::*;
    use std::panic;

    pub fn run_test<T>(val: u8, test: T) -> () 
        where T: FnOnce(LoadResult) -> () + panic::UnwindSafe
    {
        let result = panic::catch_unwind(|| {
            let logger = Arc::new(Mutex::new(log::logger::new()));
            let mem = RawMemory::new(0x8000);
            let mut cpu = Rico::new(Box::new(mem), logger);
            let oc = opcode(RefCell::new(&mut cpu));
            let lr = LoadResult::new8(val, oc);
            test(lr)
        });
        assert!(result.is_ok())
    }    

    #[test]
    fn eor_works_as_intended() 
    {
        run_test(0b11001100, |lr|
        {
            lr.origin.cpu.borrow_mut().a = 0b11110000;
            let oc = lr.xor_with_accumulator();
            assert_eq!(oc.cpu.borrow().a, 0b00111100);
        })
    }

    #[test]
    fn eor_sets_zero_if_necessary() 
    {
        run_test(0x01, |lr|
        {
            lr.origin.cpu.borrow_mut().a = 0x01;
            let oc = lr.xor_with_accumulator();
            assert_eq!(oc.cpu.borrow().status & ZERO_MASK, ZERO_MASK);
        })
    }

    #[test]
    fn ora_works_as_intended()
    {
        run_test(0b11001100, |lr|
        {
            lr.origin.cpu.borrow_mut().a = 0b11110000;
            let oc = lr.or_with_accumulator();
            assert_eq!(oc.cpu.borrow().a, 0b11111100);
        })
    }

    #[test]
    fn ora_sets_zero_if_necessary()
    {
        run_test(0b0, |lr|
        {
            lr.origin.cpu.borrow_mut().a = 0b0;
            let oc = lr.or_with_accumulator();
            assert_eq!(oc.cpu.borrow().status & ZERO_MASK, ZERO_MASK);
        })
    }

    #[test]
    fn and_works_as_intended()
    {
        run_test(0b11001100, |lr|
        {
            lr.origin.cpu.borrow_mut().a = 0b11110000;
            let oc = lr.and_with_accumulator();
            assert_eq!(oc.cpu.borrow().a, 0b11000000);
        })
    }
}
