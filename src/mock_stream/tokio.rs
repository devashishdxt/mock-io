use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::mpsc::{
        unbounded_channel as unbounded, UnboundedReceiver as Receiver, UnboundedSender as Sender,
    },
};

use crate::{error::Error, tokio::Handle};

macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            Poll::Ready(t) => t,
            Poll::Pending => return Poll::Pending,
        }
    };
}

pin_project! {
    /// Asynchronous mock IO stream
    #[derive(Debug)]
    pub struct MockStream {
        #[pin]
        read_half: ReadHalf,
        #[pin]
        write_half: WriteHalf,
    }
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
        let (sender_1, receiver_1) = unbounded();
        let (sender_2, receiver_2) = unbounded();

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

impl AsyncRead for MockStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().read_half.poll_read(cx, buf)
    }
}

impl AsyncWrite for MockStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().write_half.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().write_half.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().write_half.poll_shutdown(cx)
    }
}

/// Read half of asynchronous mock IO stream
#[derive(Debug)]
pub struct ReadHalf {
    receiver: Receiver<Vec<u8>>,
    remaining: Vec<u8>,
}

impl ReadHalf {
    async fn receive(&mut self, buf: &mut ReadBuf<'_>) -> Result<(), Error> {
        let available_space = buf.remaining();

        if self.remaining.is_empty() {
            self.remaining = self
                .receiver
                .recv()
                .await
                .ok_or(crate::error::Inner::TokioRecvError)?;
        }

        let remaining_len = self.remaining.len();

        if remaining_len > available_space {
            buf.put_slice(&self.remaining[..available_space]);
            self.remaining = self.remaining[available_space..].to_vec();

            Ok(())
        } else {
            buf.put_slice(&self.remaining);
            self.remaining = Default::default();

            Ok(())
        }
    }
}

impl AsyncRead for ReadHalf {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let mut future = Box::pin(self.get_mut().receive(buf));
        let result = ready!(future.as_mut().poll(cx));

        Poll::Ready(result.map_err(Into::into))
    }
}

/// Write half of asynchronous mock IO stream
#[derive(Debug, Clone)]
pub struct WriteHalf {
    sender: Sender<Vec<u8>>,
}

impl WriteHalf {
    fn send(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        self.sender
            .send(bytes.to_vec())
            .map(|_| bytes.len())
            .map_err(Into::into)
    }
}

impl AsyncWrite for WriteHalf {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(self.get_mut().send(buf).map_err(Into::into))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn check_stream_communication() {
        let one = 1u64.to_be_bytes().to_vec();

        let (mut sender, mut receiver) = MockStream::pair();

        assert!(matches!(sender.write(&one).await, Ok(8)));

        let mut buf = [0; 10];
        assert!(matches!(receiver.read(&mut buf).await, Ok(8)));
        assert_eq!(one[..], buf[..8]);

        assert!(matches!(sender.write(&one).await, Ok(8)));

        let mut buf = [0; 4];
        assert!(matches!(receiver.read(&mut buf).await, Ok(4)));
        assert_eq!(one[..4], buf[..]);

        let mut buf = [0; 4];
        assert!(matches!(receiver.read(&mut buf).await, Ok(4)));
        assert_eq!(one[4..], buf[..]);
    }
}
