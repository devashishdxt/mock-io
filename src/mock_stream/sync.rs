use std::{
    io::{self, Read, Write},
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::{error::Error, sync::Handle};

/// Synchronous mock IO stream
pub struct MockStream {
    read_half: ReadHalf,
    write_half: WriteHalf,
}

impl MockStream {
    /// Connects to a mock IO listener
    pub fn connect(handle: &Handle) -> Result<Self, Error> {
        let (stream_1, stream_2) = Self::pair();
        handle.send(stream_2)?;
        Ok(stream_1)
    }

    /// Creates a pair of connected mock streams
    pub fn pair() -> (Self, Self) {
        let (sender_1, receiver_1) = channel();
        let (sender_2, receiver_2) = channel();

        let stream_1 = Self {
            read_half: ReadHalf {
                receiver: receiver_1,
                remaining: Default::default(),
            },
            write_half: WriteHalf { sender: sender_2 },
        };

        let stream_2 = Self {
            read_half: ReadHalf {
                receiver: receiver_2,
                remaining: Default::default(),
            },
            write_half: WriteHalf { sender: sender_1 },
        };

        (stream_1, stream_2)
    }

    /// Splits the stream into separate read and write halves
    pub fn split(self) -> (ReadHalf, WriteHalf) {
        (self.read_half, self.write_half)
    }
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_half.read(buf)
    }
}

impl Write for MockStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_half.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.write_half.flush()
    }
}

/// Read half of synchronous mock IO stream
#[derive(Debug)]
pub struct ReadHalf {
    receiver: Receiver<Vec<u8>>,
    remaining: Vec<u8>,
}

impl ReadHalf {
    fn receive(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let available_space = buf.len();

        if self.remaining.is_empty() {
            self.remaining = self.receiver.recv()?;
        }

        let remaining_len = self.remaining.len();

        if remaining_len > available_space {
            buf.copy_from_slice(&self.remaining[..available_space]);
            self.remaining = self.remaining[available_space..].to_vec();

            Ok(available_space)
        } else {
            buf.copy_from_slice(&self.remaining[..remaining_len]);
            self.remaining = Default::default();

            Ok(remaining_len)
        }
    }
}

impl Read for ReadHalf {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.receive(buf).map_err(Into::into)
    }
}

/// Write half of synchronous mock IO stream
#[derive(Debug, Clone)]
pub struct WriteHalf {
    sender: Sender<Vec<u8>>,
}

impl WriteHalf {
    /// Sends bytes to the stream
    fn send(&self, bytes: &[u8]) -> Result<usize, Error> {
        self.sender
            .send(bytes.to_vec())
            .map(|_| bytes.len())
            .map_err(Into::into)
    }
}

impl Write for WriteHalf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.send(buf).map_err(Into::into)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_stream_communication() {
        let one = 1u64.to_be_bytes().to_vec();

        let (mut sender, mut receiver) = MockStream::pair();

        assert!(matches!(sender.write(&one), Ok(8)));

        let mut buf = [0; 8];
        assert!(matches!(receiver.read(&mut buf), Ok(8)));
        assert_eq!(one, buf[..]);

        assert!(matches!(sender.write(&one), Ok(8)));

        let mut buf = [0; 4];
        assert!(matches!(receiver.read(&mut buf), Ok(4)));
        assert_eq!(one[..4], buf[..]);

        let mut buf = [0; 4];
        assert!(matches!(receiver.read(&mut buf), Ok(4)));
        assert_eq!(one[4..], buf[..]);
    }
}
