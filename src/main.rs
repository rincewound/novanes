mod core6502;
mod memory;

fn main() {
    let m = memory::RawMemory::new(0x8000);
    let mut core = core6502::Rico::new(Box::new(m));

    loop
    {
        core.execute(100);
    }

    //println!("Hello, world!");
}
