
use crate::core6502::*;

use std::cell::RefCell;


pub struct Opcode<'a>
{
    cpu: RefCell <&'a mut crate::core6502::Rico>
}

pub enum RegisterName
{
    A,
    X,
    Y,
    PC,
    S,
    Status
}

pub struct LoadResult<'a>
{
    val: u16,
    origin: Opcode<'a>
}

impl<'a> LoadResult<'a>
{ 
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
        match target{
            RegisterName::A => self.origin.cpu.borrow_mut().a = self.val as u8,
            RegisterName::X => self.origin.cpu.borrow_mut().x = self.val as u8,
            RegisterName::Y => self.origin.cpu.borrow_mut().y = self.val as u8,
            RegisterName::PC => self.origin.cpu.borrow_mut().pc = self.val,
            RegisterName::S => self.origin.cpu.borrow_mut().s = self.val as u8,
            RegisterName::Status => self.origin.cpu.borrow_mut().status = self.val as u8,
        }
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
}

impl<'a> Opcode<'a>
{
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

    pub fn has_mnemonic(self, nmonic: String ) -> Opcode<'a>
    {
        self.cpu.borrow_mut().last_opcode_nmonic = nmonic;
        self
    }

    fn load_u16(&self, adr: u16) -> u16
    {
        let cpu = self.cpu.borrow();
        let hi: u16 = cpu.mem.read_byte(adr as usize).unwrap() as u16;
        let lo: u16 = cpu.mem.read_byte((adr + 1) as usize).unwrap() as u16;
        let res = lo | (hi << 8);
        res
    }

    fn fetch_u8(&self, adr: u16) -> u8
    {
        let cpu = self.cpu.borrow();
        cpu.mem.read_byte(adr as usize).unwrap()
    }

    fn load_u8_from_mem(self, adr: u16) -> LoadResult<'a>
    {
        let result = self.cpu.borrow().mem.read_byte(adr as usize);
        let pc = self.cpu.borrow().pc;

        match result
        {
            Ok(val) => LoadResult::new8(val, self),
            Err(_) => panic!("access violation {:#4x}", pc + 1)
        }  
    }

    fn read_register(&self, reg: RegisterName) -> u8
    {
        match reg
        {
            RegisterName::A => self.cpu.borrow().a,
            RegisterName::X => self.cpu.borrow().x,
            RegisterName::Y => self.cpu.borrow().y,
            RegisterName::S => self.cpu.borrow().s,
            RegisterName::Status => self.cpu.borrow().status,
            _ => panic!("cannot read this register as 8 bit value")
        }
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
            let adrread = self.cpu.borrow().mem.read_byte(adr as usize);
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

}

pub fn opcode(cpu: RefCell <&mut crate::core6502::Rico>) -> Opcode
{
    Opcode::new(cpu)
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
            let mem = RawMemory::new(0x8000);
            let mut cpu = Rico::new(Box::new(mem));
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
