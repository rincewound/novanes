use crate::memory::Memory;
use std::cell::RefCell;
use std::sync::Arc;

pub struct SpriteDMA
{
    ram: Arc<RefCell<crate::memory::CompositeMemory>>
}

impl Memory for SpriteDMA
{
    fn read_byte(&mut self, address: usize) -> Result<u8, crate::memory::MemError> {
        todo!()
    }

    fn write_byte(&mut self, address: usize, data: u8) -> crate::memory::MemError {
        let target_address = data as u16 * 0x100 as u16;
        crate::memory::MemError::Ok
    }
}