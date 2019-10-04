
extern crate queues;

use queues::*;

pub struct logger{
    buf: queues::CircularBuffer<String>
}

impl logger{
    pub fn new() -> Self
    {
        logger {buf : queues::CircularBuffer::<String>::new(35) }
    }
    pub fn write(&mut self, message: String)
    {
        self.buf.add(message);
    }

    pub fn to_console(&mut self)
    {
        let mut done: bool = false;
        while !done{
            let val = self.buf.remove();
            match val
            {
                Ok(txt) => println!("{}", txt),
                Err(_) => done = true
            }
        }
    }
}
