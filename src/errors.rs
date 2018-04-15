use std::convert::From;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "I/O error")]
    Io(#[cause] ::std::io::Error),

    #[fail(display = "Invalid protocol revision number: {}", _0)]
    InvalidProtocolRevisionNumber(u8),

    /// Invalid information element identifier.
    #[fail(display = "Invalid information element identifier: {}", _0)]
    InvalidInformationElementIdentifier(u8),

    /// The timestamp is negative, but only positive ones are supported.
    #[fail(display = "Negative timestamp: {}", _0)]
    NegativeTimestamp(i64),

    /// The overall message length is too long.
    #[fail(display = "The overall message length is too long: {}", _0)]
    OverallMessageLength(usize),

    /// The payload is too long.
    #[fail(display = "The payload is too long: {}", _0)]
    PayloadTooLong(usize),

    /// No headers in a message
    #[fail(display = "NoHeaders headers in a message")]
    NoHeader,

    /// No payloads in an message.
    #[fail(display = "Two payloads in a MO message")]
    NoPayload,

    /// Two headers in a message
    #[fail(display = "Two headers in a message")]
    TwoHeaders,

    /// Two payloads in an message.
    #[fail(display = "Two payloads in a MO message")]
    TwoPayloads,

    /// The session status is unknown.
    #[fail(display = "Unexpected EOF during reading {}.", _0)]
    UnknownSessionStatus(u8),
}

/// Create-specific `Result`.
pub type Result<T> = ::std::result::Result<T, Error>;

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error {
        Error::Io(err)
    }
}
