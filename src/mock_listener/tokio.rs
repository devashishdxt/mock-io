use tokio::sync::mpsc::{
    unbounded_channel as unbounded, UnboundedReceiver as Receiver, UnboundedSender as Sender,
};

use crate::{error::Error, tokio::MockStream};

#[derive(Debug)]
/// Asynchronous mock IO listener
pub struct MockListener {
    receiver: Receiver<MockStream>,
}

impl MockListener {
    /// Creates a new asynchronous mock listener
    pub fn new() -> (Self, Handle) {
        let (sender, receiver) = unbounded();

        (Self { receiver }, Handle { sender })
    }

    /// Accept a new connection. Returns a mock stream supplied by the sender
    pub async fn accept(&mut self) -> Result<MockStream, Error> {
        self.receiver
            .recv()
            .await
            .ok_or(crate::error::Inner::TokioRecvError)
            .map_err(Into::into)
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
