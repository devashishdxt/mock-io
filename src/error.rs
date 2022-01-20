//! Error types used in this crate
use std::{error::Error as IError, fmt, sync::mpsc, io};

use thiserror::Error;

/// Error returned by functions in this crate
#[derive(Debug)]
pub struct Error {
    inner: Inner,
}

impl Error {
    /// Returns the kind of error
    pub fn kind(&self) -> ErrorKind {
        (&self.inner).into()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind().fmt(f)
    }
}

impl IError for Error {
    fn source(&self) -> Option<&(dyn IError + 'static)> {
        self.inner.source()
    }
}

#[doc(hidden)]
impl From<Error> for std::io::Error {
    fn from(error: Error) -> Self {
        match error {
            Error { inner: Inner::SyncRecvTimeoutError(mpsc::RecvTimeoutError::Timeout) } => {
                io::Error::new(io::ErrorKind::TimedOut, "timed out")
            }
            other => std::io::Error::new(std::io::ErrorKind::Other, other)
        }
    }
}

#[doc(hidden)]
impl<T: Into<Inner>> From<T> for Error {
    fn from(inner: T) -> Self {
        Self {
            inner: inner.into(),
        }
    }
}

/// Different kinds of possible errors returned by functions in this crate
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Stream connecting error
    StreamConnectError,
    /// Channel receiving error
    ChannelRecvError,
    /// Channel receiving error with timeout
    ChannelRecvTimeoutError,
    /// Channel sending error
    ChannelSendError,
    /// Other error
    Other,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StreamConnectError => write!(f, "Stream connecting error"),
            Self::ChannelRecvError => write!(f, "Channel receiving error"),
            Self::ChannelRecvTimeoutError => write!(f, "Channel receiving error"),
            Self::ChannelSendError => write!(f, "Channel sending error"),
            Self::Other => write!(f, "Other error"),
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Error)]
pub enum Inner {
    #[cfg(feature = "sync")]
    #[error("Sync stream connecting error")]
    SyncConnectError,
    #[cfg(feature = "sync")]
    #[error("Sync channel receiving error: {0}")]
    SyncRecvError(#[from] std::sync::mpsc::RecvError),
    #[cfg(feature = "sync")]
    #[error("Sync channel with timeout receiving error: {0}")]
    SyncRecvTimeoutError(#[from] std::sync::mpsc::RecvTimeoutError),
    #[cfg(feature = "sync")]
    #[error("Sync channel sending error: {0}")]
    SyncSendError(#[from] std::sync::mpsc::SendError<Vec<u8>>),
    #[cfg(feature = "async-futures")]
    #[error("Async stream connecting error")]
    AsyncConnectError(#[from] async_channel::SendError<crate::futures::MockStream>),
    #[cfg(feature = "async-futures")]
    #[error("Async channel receiving error: {0}")]
    AsyncRecvError(#[from] async_channel::RecvError),
    #[cfg(feature = "async-futures")]
    #[error("Async channel sending error: {0}")]
    AsyncSendError(#[from] async_channel::SendError<Vec<u8>>),
    #[cfg(feature = "async-tokio")]
    #[error("Tokio stream connecting error")]
    TokioConnectError(#[from] tokio::sync::mpsc::error::SendError<crate::tokio::MockStream>),
    #[cfg(feature = "async-tokio")]
    #[error("Tokio channel receiving error")]
    TokioRecvError,
    #[cfg(feature = "async-tokio")]
    #[error("Tokio channel sending error: {0}")]
    TokioSendError(#[from] tokio::sync::mpsc::error::SendError<Vec<u8>>),
    #[error("Other error")]
    Other,
}

impl<'a> From<&'a Inner> for ErrorKind {
    fn from(inner: &'a Inner) -> Self {
        match inner {
            #[cfg(feature = "sync")]
            Inner::SyncConnectError => ErrorKind::StreamConnectError,
            #[cfg(feature = "sync")]
            Inner::SyncRecvError(_) => ErrorKind::ChannelRecvError,
            #[cfg(feature = "sync")]
            Inner::SyncRecvTimeoutError(_) => ErrorKind::ChannelRecvTimeoutError,
            #[cfg(feature = "sync")]
            Inner::SyncSendError(_) => ErrorKind::ChannelSendError,
            #[cfg(feature = "async-futures")]
            Inner::AsyncConnectError(_) => ErrorKind::StreamConnectError,
            #[cfg(feature = "async-futures")]
            Inner::AsyncRecvError(_) => ErrorKind::ChannelRecvError,
            #[cfg(feature = "async-futures")]
            Inner::AsyncSendError(_) => ErrorKind::ChannelSendError,
            #[cfg(feature = "async-tokio")]
            Inner::TokioConnectError(_) => ErrorKind::StreamConnectError,
            #[cfg(feature = "async-tokio")]
            Inner::TokioRecvError => ErrorKind::ChannelRecvError,
            #[cfg(feature = "async-tokio")]
            Inner::TokioSendError(_) => ErrorKind::ChannelSendError,
            Inner::Other => ErrorKind::Other,
        }
    }
}

#[cfg(feature = "sync")]
impl From<std::sync::mpsc::SendError<crate::sync::MockStream>> for Inner {
    fn from(_: std::sync::mpsc::SendError<crate::sync::MockStream>) -> Self {
        Self::SyncConnectError
    }
}
