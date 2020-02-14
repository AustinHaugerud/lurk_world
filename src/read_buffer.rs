use std::io::{Read, Error};
use std::net::TcpStream;

pub struct ReadBuffer {
    source: TcpStream,
    data: Vec<u8>,
}

impl Read for ReadBuffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.source.read(buf)
    }
}

impl ReadBuffer {
    pub fn buffer(&self) -> &[u8] {
        &self.data
    }

    pub fn fill_buf(&mut self) -> Result<usize, Error> {
        let mut temp = Vec::with_capacity(std::u16::MAX as usize);
        let read = self.read(&mut temp)?;
        self.data.append(&mut temp);
        Ok((read))
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl From<TcpStream> for ReadBuffer {
    fn from(stream: TcpStream) -> Self {
        Self {
            source: stream,
            data: vec![],
        }
    }
}
