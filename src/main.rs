mod rico;

fn main() {
    let m = rico::memory::RawMemory::new(0x8000);
    let mut core = rico::Rico::new(Box::new(m));

    loop
    {
        core.execute(100);
    }

    //println!("Hello, world!");
}
