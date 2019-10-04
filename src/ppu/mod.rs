use crate::memory::*;

const PIXELS_PER_SCANLINE: u16 = 256;
const VISIBLE_SCANLINES : u16 = 224;
const OVERSCAN: u16 = 240;
const VLBANKEND: u16 = VISIBLE_SCANLINES + 20;

const VBlankBit: u8   = 0b10000000;
const Sprite0Occ: u8  = 0b01000000;
const ScanSprCnt: u8  = 0b00100000;
const VRAMWrite: u8   = 0b00010000; 

const pixels_per_tick : u16 = 3;

pub struct ppu
{
    status: u8,
    line: u16,
    lastpixel: u16,
}

impl ppu
{
    pub fn new() -> Self
    {
        ppu {
            status: 0,
            line: 0,
            lastpixel: 0
            }
    }
}

impl Memory for ppu
{
    fn read_byte(&mut self, address: usize) -> Result<u8, MemError>
    {
        match address
        {
            0x02 => {
                println!("          PPU.ReadStatus: #({:#02x})", self.status);
               let statuscopy = self.status;
               self.status &= !VBlankBit;
               return Ok(statuscopy)
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
        // let err = format!("         PPU.Write: {:#04x} -> Bad Addr", address);
        // println!("{}", err);
        // MemError::BadAddress
    }

    fn tick(&mut self, clock_ticks: u32)
    {
        // the cpu has run clock_ticks cycles. The ppu may now draw
        // clock_ticks * 3 pixels
        self.lastpixel += clock_ticks as u16 * pixels_per_tick;
        if self.lastpixel > PIXELS_PER_SCANLINE
        {
            self.line +=1;
            self.lastpixel = self.lastpixel % PIXELS_PER_SCANLINE;

            if self.line > VLBANKEND 
            {
                self.line = 0;
                self.status &= !VBlankBit;
            }

            if self.line > VISIBLE_SCANLINES
            {
                self.status |= VBlankBit;                
            }

        }

        
    }
}