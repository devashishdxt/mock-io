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
