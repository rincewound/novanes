use crate::memory::{Memory, MemTickResult};
use std::cell::RefCell;
use std::rc::Rc;

pub struct SpriteDMA
{
    ram: Rc<RefCell<crate::memory::CompositeMemory>>
}

impl SpriteDMA
{
    pub fn new(ram: Rc<RefCell<crate::memory::CompositeMemory>>) -> Self
    {
        Self {ram}
    }
}

impl Memory for SpriteDMA
{
    fn read_byte(&mut self, address: usize) -> Result<u8, crate::memory::MemError> {
        todo!()
    }

    fn write_byte(&mut self, address: usize, data: u8) -> crate::memory::MemError {
        let data_source_address = data as u16 * 0x100 as u16;
        let mut mem = self.ram.borrow_mut();
        for i in 0..256
        {
            if let Ok(source_byte) = mem.read_byte((data_source_address + i) as usize)
            {
                mem.write_byte(0x2004, source_byte);
            }
            
        }
        crate::memory::MemError::Ok
    }

    fn tick(&mut self, _clock_ticks: u32) -> crate::memory::MemTickResult {
        MemTickResult::Ok
    }
}