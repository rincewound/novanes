mod core6502;
mod memory;
mod ppu;
mod log;
mod dma;

extern crate minifb;

use minifb::{Key, Window, WindowOptions};

use std::io::{self, BufReader, Read};
use std::fs::{self, File};
use std::path::Path;

use std::sync::Mutex;
use std::{slice, sync::Arc, cell::RefCell};

struct INESHeader
{
    const_data: [u8; 4],     // always "NES/0x1A"
    prog_rom_banks: u8,       // in 16 KiB units
    charRomBanks: u8,       // in 8 KiB units
    flagBytes: [u8; 10]
}

fn read_struct<T, R: Read>(read: &mut R) -> io::Result<T> {
    let num_bytes = ::std::mem::size_of::<T>();
    unsafe {
        let mut s = ::std::mem::uninitialized();
        let buffer = slice::from_raw_parts_mut(&mut s as *mut T as *mut u8, num_bytes);
        match read.read_exact(buffer) {
            Ok(()) => Ok(s),
            Err(e) => {
                ::std::mem::forget(s);
                Err(e)
            }
        }
    }
}

fn load_rom(romfile: String, targetMemory: &mut dyn memory::Memory)
{
    println!("Open {} ", romfile);
    let file = fs::File::open(romfile);
    let mut rd = BufReader::new(file.unwrap());
    let res = read_struct::<INESHeader,_>(&mut rd).unwrap();    
    println!("Has {} PRG ROM banks", res.prog_rom_banks);
        
    let mut prg_buf : [u8; 16384] = [0x00; 16384];

    let mut x: u32;
    for x in 0..res.prog_rom_banks
    {        
        rd.read_exact(&mut prg_buf);
        let mut i: u32 = 0;
        while i < 16384
        {
            let actualadr = (i + (x as u32) * 16384) as usize;
            targetMemory.write_byte(actualadr, prg_buf[i as usize]);
            i += 1;
        }
    }

}

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

fn make_window() -> Window
{

    let mut window = Window::new(
        "NOVANES - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    window
}

fn main() 
{
    let fb = Arc::new(RefCell::new(vec![0u32; WIDTH * HEIGHT]));

    let mut window = make_window();

    let logger = Arc::new(Mutex::new(log::logger::new()));
    let ppu = ppu::ppu::new(logger.clone(), fb.clone());
    let ram = memory::RawMemory::new(0x2000);
    let mut m = memory::RawMemory::new(0x8000);
    load_rom("./roms/smb1.nes".to_string(), &mut m);
    let mut memmap = memory::CompositeMemory::new();

    let memmorycell = Arc::new(RefCell::new(memmap));

    // ToDo: Add peripherals as ranges as well.
    memmap.register_range(0x0000, 0x1FFF, Box::new(ram));
    memmap.register_range(0x8000, 0x8000 + 0x8000, Box::new(m));
    memmap.register_range(0x2000, 0x2000 + 0x0008, Box::new(ppu));    
    memmap.register_range(0x4014, 0x4014, Box::new(dma::SpriteDMA{}));    
    let mut core = core6502::Rico::new(Box::new(memmap), logger.clone());

    while window.is_open() && !window.is_key_down(Key::Escape) 
    {
        // Do CPU ticks for a complete frame.
        // the memtick will cause the ppu to draw as well.
        for _ in 0..240
        {        
            // NTSC has 113 2/3 cycles per scanline, we round this to 114, this
            // boils down to three pixels per CPU cycle. We sync once per
            // scanline
            core.execute(114);
        }

        // When 240 scanlines are done, we display the current frame.
        window
            .update_with_buffer(fb.borrow().as_slice(), WIDTH, HEIGHT)
            .unwrap();
    }

}
