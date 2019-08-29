mod core6502;

fn main() {
    let m = core6502::memory::RawMemory::new(0x8000);
    let mut core = core6502::Rico::new(Box::new(m));

    loop
    {
        core.execute(100);
    }

    //println!("Hello, world!");
}
