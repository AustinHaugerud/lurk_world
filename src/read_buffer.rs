use std::io::{Read, Error};
use std::net::TcpStream;

pub struct ReadBuffer {
    source: TcpStream,
    data: Vec<u8>,
}

impl Read for ReadBuffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let num = self.data.len().min(buf.len());
        let take: Vec<u8>  = self.data.drain(0..num).collect();
        println!("New length is {}.", self.data.len());
        buf[0..num].copy_from_slice(&take[..]);
        Ok(num)
    }
}

impl ReadBuffer {
    pub fn buffer(&self) -> &[u8] {
        &self.data
    }

    pub fn fill_buf(&mut self) -> Result<usize, Error> {
        let mut temp = [0u8; 32_000];
        let read = self.source.read(&mut temp)?;
        self.data.append(&mut temp[..read].to_vec());
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
