
#[derive(Debug, PartialEq)]
pub enum MemError
{
    Ok,
    BadAddress
}

pub trait Memory
{
    fn read_byte(&mut self, address: usize) -> Result<u8, MemError>;
    fn write_byte(&mut self, address: usize, data: u8) -> MemError;
    fn read_u16(&mut self, address: usize) -> Result<u16, MemError>
    {
        let hi = self.read_byte(address);
        let lo = self.read_byte(address + 1);

        match hi
        {
            Ok(v) => {
                match lo 
                {
                    Ok(v2) => Ok((v as u16) | ((v2 as u16) << 8) ),
                    _ => Err(MemError::BadAddress)
                }            
            },
            _ => Err(MemError::BadAddress)
        }

        //let res = lo | (hi << 8);
        //res
    }

    fn tick(&mut self, clock_ticks: u32){}
}

pub struct RawMemory
{
    data: Vec<u8>
}

struct AddressRange
{
    begin: usize,
    end: usize
}

pub struct CompositeMemoryEntry
{
    range: AddressRange,
    handler: Box<dyn Memory>
}

pub struct CompositeMemory
{
    handlers: Vec< Box<CompositeMemoryEntry> >
}

impl Memory for CompositeMemory
{
    fn read_byte(&mut self, address: usize) -> Result<u8, MemError>
    {
        // find correct handler:
        let mut it = self.handlers.iter_mut();

        let mut m = it.find(|x| x.range.begin <= address && x.range.end >= address);

        if let Some(m) =  m {
            let rangestart = m.range.begin;
            return m.handler.read_byte(address - rangestart);
        }
        Err(MemError::BadAddress)    
    }

    fn write_byte(&mut self, address: usize, data: u8) -> MemError
    {
        // find correct handler:
        let mut it = self.handlers.iter_mut();

        let m = it.find(|x| x.range.begin <= address && x.range.end >= address);

        if let Some(m) = m {
            let err = format!("          {:#4x} -> {:#2x}", address, data);
            println!("{}", err);
            return m.handler.write_byte(address, data);
        }

        let err = format!("Memory.WriteByte: {:#4x} -> Bad Addr", address);
        println!("{}", err);
        MemError::BadAddress
    }

    fn tick(&mut self, clock_ticks: u32)
    {
        let it = self.handlers.iter_mut();
        for m in it
        {
            m.handler.tick(clock_ticks);
        }
    }
}

impl CompositeMemory
{
    pub fn new() -> Self
    {
        CompositeMemory {handlers: vec!()}
    }

    pub fn register_range(&mut self, begin: usize, end: usize, mem:  Box<dyn Memory>)
    {
        let entry = CompositeMemoryEntry {
            range : AddressRange{begin: begin, end: end},
            handler: mem
        };

        self.handlers.push( Box::new(entry) );
    }
}

impl Memory for RawMemory
{
    fn read_byte(&mut self, address: usize) -> Result<u8, MemError>
    {
        if address < self.data.len()
        {
            return Ok(self.data[address]);
        }        
        let err = format!("RawMemory.Read: {:#04x} -> Bad Addr", address);
        println!("{}", err);
        Err(MemError::BadAddress)
    }

    fn write_byte(&mut self, address: usize, data: u8) -> MemError
    {
        if address < self.data.len()
        {
            self.data[address] = data;
            return MemError::Ok;
        }
        let err = format!("RawMemory.Write: {:#04x} -> Bad Addr", address);
        println!("{}", err);
        MemError::BadAddress
    }
}

impl RawMemory
{
    fn fill(&mut self, _data: u8)
    {
        for _ in 0..self.data.capacity()  
        {
            self.data.push(_data);            
        }
    }

    pub fn new(size: usize) -> Self
    {
        let mut mem = RawMemory
        {
            data : Vec::with_capacity(size)
        };
        mem.fill(0x00);
        mem
    }
}

#[test]
fn rawmem_can_write() 
{
    let mut m = RawMemory::new(0x8000);

    m.write_byte(0x4004, 0xFF);

    let read = m.read_byte(0x4004).unwrap();

    assert_eq!(0xFF, read );
}


#[test]
fn rawmem_write_out_of_bounds_fails_gracefully()
{
    let mut m = RawMemory::new(0x4000);

    let res = m.write_byte(0x4004, 0xFF);

    assert_eq!(MemError::BadAddress, res);
}

#[test]
fn compositemem_dispatches_memory_correctly()
{
    let mut m =  CompositeMemory::new();
    let r = RawMemory::new(0x4000);

    m.register_range(0x1000, 0x5000, Box::new(r));
    m.write_byte(0x1000, 0xFA);
    let read = m.read_byte(0x1000).unwrap();

    assert_eq!(0xFA, read );
}

#[test]
fn compositemem_write_to_invalid_range_fails_gracefully()
{
    let mut m =  CompositeMemory::new();
    let r = RawMemory::new(0x4000);

    m.register_range(0x1000, 0x5000, Box::new(r));
    let res = m.write_byte(0x21, 0xFA);
    assert_eq!(MemError::BadAddress, res);
}