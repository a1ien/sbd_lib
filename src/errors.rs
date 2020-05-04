use std::convert::From;

#[derive(Debug)]
pub enum Error {
    /// I/O error
    Io(::std::io::Error),

    /// Invalid protocol revision number
    InvalidProtocolRevisionNumber(u8),

    /// Invalid information element identifier.
    InvalidInformationElementIdentifier(u8),

    /// The timestamp is negative, but only positive ones are supported.
    NegativeTimestamp(i64),

    /// The overall message length is too long.
    OverallMessageLength(usize),

    /// The payload is too long.
    PayloadTooLong(usize),

    /// No headers in a message
    NoHeader,

    /// No payloads in an message.
    NoPayload,

    /// Two headers in a message
    TwoHeaders,

    /// Two payloads in an message.
    TwoPayloads,

    /// Two locations in an message.
    TwoLocations,

    /// The session status is unknown.
    UnknownSessionStatus(u8),
}

/// Create-specific `Result`.
pub type Result<T> = ::std::result::Result<T, Error>;

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error {
        Error::Io(err)
    }
}
