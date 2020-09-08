use crate::memory::*;
use crate::log;
use std::{cell::RefCell, sync::{Arc,Mutex}};


const PIXELS_PER_SCANLINE: u16 = 256;
const VISIBLE_SCANLINES : u16 = 224;
const OVERSCAN: u16 = 240;
const VLBANKEND: u16 = VISIBLE_SCANLINES + 20;

const VBlankBit: u8   = 0b10000000;
const Sprite0Occ: u8  = 0b01000000;
const ScanSprCnt: u8  = 0b00100000;
const VRAMWrite: u8   = 0b00010000; 

const VRamAdrIncBit : u8 = 0b00000010;

const pixels_per_tick : u16 = 3;

pub struct ppu
{
    ctrl0: u8,
    status: u8,
    line: u16,
    lastpixel: u16,
    vramadrbyte1: bool,
    vramadr: u16,
    vram: [u8; 0x3FFF],
    logger: Arc<Mutex<log::logger>>,
    framebuffer: Arc<RefCell<Vec<u32>>>   
}

impl ppu
{
    pub fn new(log: Arc<Mutex<log::logger>>, framebuffer: Arc<RefCell<Vec<u32>>>) -> Self
    {
        ppu {
            ctrl0: 0x00,
            status: 0,
            line: 0,
            lastpixel: 0,
            vramadrbyte1: false,
            vramadr: 0x000,
            vram: [0; 0x3FFF],
            logger: log,
            framebuffer: framebuffer
            }
    }

    pub fn log(&self, message: String)
    {
        let mut lg = self.logger.lock().unwrap();
        lg.write(message);
    }

    fn get_nametable_index(line: u32, pixel: u32) -> u32
    {
        let x = pixel / 32;
        let y= line / 32;
        return y * 32 + x;
    }
}

impl Memory for ppu
{
    fn read_byte(&mut self, address: usize) -> Result<u8, MemError>
    {
        match address
        {
            0x02 => {
               self.log(format!("          PPU.ReadStatus: #({:#02x})", self.status));
               let statuscopy = self.status;
               //self.status &= !VBlankBit;
               return Ok(statuscopy)
            },
            _ => {}
        }

        let err = format!("PPU.Read: {:#04x} -> Bad Addr", address);
        self.log(format!("{}", err));
        Err(MemError::BadAddress)
    }

    fn write_byte(&mut self, address: usize, data: u8) -> MemError
    {
        let actual_address = address + 0x2000;

        match actual_address
        {
            0x2000 => {
                self.log(format!("          PPU.Ctrl1 -> {:#2x}", data));
                return MemError::Ok;
            },
            0x2001 => {
                self.log(format!("          PPU.Ctrl2 -> {:#2x}", data));
                return MemError::Ok;
            }
            0x2002 => {
                panic!("Cannot write to PPU.Status");
            },
            0x2003 => {
                self.log(format!("          PPU.OAMADR -> {:#2x}", data));                
                return MemError::Ok;
            },
            0x2004 =>
            {
                panic!("no OAM support yet.");
            },
             0x2005=> {
                 // Note: This comes as two values , one for x one for y
                self.log(format!("          PPU.Scroll -> {:#2x}", data));
                return MemError::Ok;
            },
            0x2006 => {
                self.log(format!("          PPU.Addr -> {:#2x}", data));
                let datau16 = data as u16;
                if !self.vramadrbyte1
                {
                    self.vramadr = (self.vramadr & 0xFF00) | datau16 ;                        
                }
                else
                {
                    let datashift = datau16 << 8;
                    self.vramadr = (self.vramadr & 0xFF) | datashift;
                }
                let logmsg = format!("          PPU.VRAMADR -> {:#2x}", self.vramadr);
                self.log(logmsg);                
                self.vramadrbyte1 = !self.vramadrbyte1;
                return MemError::Ok;
            },
            0x2007 => {
                self.log(format!("          PPU.Data -> {:#2x}", data));
                self.log(format!("          PPU.VRAM {:#2x} -> {:#2x}", self.vramadr, data));

                if self.vramadr > 0x3FFF
                {
                    self.logger.lock().unwrap().to_console();
                    panic!("Invalid VRAM access");
                }

                self.vram[self.vramadr as usize] = data;
                let increment = if self.ctrl0 & VRamAdrIncBit == 0 { 1 } else { 32 };
                self.vramadr += increment;
                if self.vramadr >= 0x3FFF
                {
                    self.vramadr -= 0x3FFF;
                }


                return MemError::Ok;
            },

            0x4014 => {
                panic!("OAM: Implement me :)")
            },

            _ => panic!(format!("Invalid address: {:#4x} ", actual_address))
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



    fn tick(&mut self, clock_ticks: u32) -> MemTickResult
    {
        let num_pixels_to_draw = clock_ticks as u32 * pixels_per_tick as u32;
        let mut framebuffer = self.framebuffer.borrow_mut();
        let slice = framebuffer.as_mut_slice();
        let start_index = self.line as u32 * PIXELS_PER_SCANLINE as u32 + self.lastpixel as u32;

        

        for i in start_index.. start_index + num_pixels_to_draw
        {
            // step 1: calculate nametable index:
            let nametable_index = ppu::get_nametable_index(self.line as u32, i);
            // Fetch a nametable entry from $2000-$2FBF.
            let nametable_entry = self.vram[0x2000 + nametable_index as usize];
            // Fetch the corresponding attribute table entry from $23C0-$2FFF and increment the current VRAM address within the same row.
            // Fetch the low-order byte of an 8x1 pixel sliver of pattern table from $0000-$0FF7 or $1000-$1FF7.
            // Fetch the high-order byte of this sliver from an address 8 bytes higher.
            // Turn the attribute data and the pattern table data into palette indices, and combine them with data from sprite data using priority.


            slice[(i) as usize] = nametable_entry as u32;
        }

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
                let status_cpy = self.status;
                self.status |= VBlankBit;                 
                if(status_cpy & VBlankBit) == 0
                {
                    // Just entered VBlank, generate NMI.
                    return MemTickResult::IRQ( 0b001 as u8)
                }
            }

        }
        MemTickResult::Ok        
    }
}

// #[cfg(test)]
// mod pputests 
// {
//     use crate::ppu::*;

//     #[test]
//     pub fn write_data_writes_data_correctly()
//     {
//         let mut p = ppu::new(Arc::new(Mutex::new(log::logger::new())));
//         p.write_byte(0x0006, 0x21);
//         p.write_byte(0x0006, 0x08);
//         assert_eq!(0x2108, p.vramadr);
//     }
//}