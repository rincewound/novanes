use crate::memory::*;

pub struct ppu
{
    status: u8
}

impl ppu
{
    pub fn new() -> Self
    {
        ppu {status: 0}
    }
}

impl Memory for ppu
{
    fn read_byte(&self, address: usize) -> Result<u8, MemError>
    {
        match address
        {
            0x02 => {
                println!("          PPU.ReadStatus: #({:#02x})", self.status);
               return Ok(self.status)
            },
            _ => {}
        }

        let err = format!("PPU.Read: {:#04x} -> Bad Addr", address);
        println!("{}", err);
        Err(MemError::BadAddress)
    }

    fn write_byte(&mut self, address: usize, data: u8) -> MemError
    {
        match address
        {
            0x2000 => {
                println!("          PPU.Ctrl1 -> {:#2x}", data);
                return MemError::Ok;
            },
            0x2001 => {
                println!("          PPU.Ctrl2 -> {:#2x}", data);
                return MemError::Ok;
            }
            0x2002 => {
                panic!("Cannot write to PPU.Status");
            },
            _ => panic!("Invalid address")
        }

        // if address < self.data.len()
        // {
        //     self.data[address] = data;
        //     return MemError::Ok;
        // }
        let err = format!("         PPU.Write: {:#04x} -> Bad Addr", address);
        println!("{}", err);
        MemError::BadAddress
    }

    fn tick(&mut self)
    {
        
    }
}