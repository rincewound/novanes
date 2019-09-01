mod core6502;
mod memory;

use std::io::{self, BufReader, Read};
use std::fs::{self, File};
use std::path::Path;
use std::slice;

struct INESHeader
{
    constData: [u8; 4],     // always "NES/0x1A"
    progRomBanks: u8,       // in 16 KiB units
    charRomBanks: u8,       // in 8 KiB units
    flagBytes: [u8; 10]
}

fn read_struct<T, R: Read>(mut read: &mut R) -> io::Result<T> {
    let num_bytes = ::std::mem::size_of::<T>();
    unsafe {
        let mut s = ::std::mem::uninitialized();
        let mut buffer = slice::from_raw_parts_mut(&mut s as *mut T as *mut u8, num_bytes);
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
    println!("Has {} PRG ROM banks", res.progRomBanks);
        
    let mut prg_buf : [u8; 16384] = [0x00; 16384];

    let mut x: u32;
    for x in 0..res.progRomBanks
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

fn main() {
    let mut m = memory::RawMemory::new(0x8000);


    let mut core = core6502::Rico::new(Box::new(m));

    loop
    {
        core.execute(100);
    }

    //println!("Hello, world!");
}
