use std::sync::mpsc::{channel, Receiver, Sender};

use crate::{error::Error, sync::MockStream};

#[derive(Debug)]
/// Synchronous mock IO listener
pub struct MockListener {
    receiver: Receiver<MockStream>,
}

impl MockListener {
    /// Creates a new synchronous mock listener
    pub fn new() -> (Self, Handle) {
        let (sender, receiver) = channel();

        (Self { receiver }, Handle { sender })
    }

    /// Accept a new connection. Returns a mock stream supplied by the sender
    pub fn accept(&self) -> Result<MockStream, Error> {
        self.receiver.recv().map_err(Into::into)
    }
}

/// Handle for synchronous mock IO listener used to connect to the listener
pub struct Handle {
    sender: Sender<MockStream>,
}

impl Handle {
    pub(crate) fn send(&self, mock_stream: MockStream) -> Result<(), Error> {
        self.sender.send(mock_stream).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        io::{Read, Write},
        thread,
    };

    #[test]
    fn check_listener_flow() {
        let (listener, handle) = MockListener::new();

        thread::spawn(move || {
            let mut stream = MockStream::connect(&handle).unwrap();
            stream.write(&1u64.to_be_bytes()).unwrap();
            stream.write(&2u64.to_be_bytes()).unwrap();
        });

        while let Ok(mut stream) = listener.accept() {
            let mut buf = [0; 8];

            stream.read(&mut buf).unwrap();
            assert_eq!(1u64.to_be_bytes(), buf);

            stream.read(&mut buf).unwrap();
            assert_eq!(2u64.to_be_bytes(), buf);
        }
    }
}
