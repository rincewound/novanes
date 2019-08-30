
#[derive(Debug, PartialEq)]
pub enum MemError
{
    Ok,
    BadAddress
}

pub trait Memory
{
    fn read_byte(&self, address: usize) -> Result<u8, MemError>;
    fn write_byte(&mut self, address: usize, data: u8) -> MemError;
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
    fn read_byte(&self, address: usize) -> Result<u8, MemError>
    {
        // find correct handler:
        let mut it = self.handlers.iter();

        let m = it.find(|x| x.range.begin <= address && x.range.end >= address);

        if let Some(m) = m {
            return m.handler.read_byte(address);
        }
        Err(MemError::BadAddress)    
    }

    fn write_byte(&mut self, address: usize, data: u8) -> MemError
    {
        // find correct handler:
        let mut it = self.handlers.iter_mut();

        let m = it.find(|x| x.range.begin <= address && x.range.end >= address);

        if let Some(m) = m {
            let err = format!("{:x} -> {:x}", address, data);
            print!("{}", err);
            return m.handler.write_byte(address, data);
        }

        let err = format!("{:x} -> Bad Addr", address);
        println!("{}", err);
        MemError::BadAddress
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
    fn read_byte(&self, address: usize) -> Result<u8, MemError>
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