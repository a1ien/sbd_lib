use crate::Result;
use byteorder::WriteBytesExt;
use std::io::{Read, Write};

/// The Message Delivery Confirmation of a mobile-originated session.
///
/// The descriptions for these codes are taken directly from the `DirectIP` documentation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ConfirmationStatus {
    /// The SBD session completed successfully.
    status: bool,
}

impl ConfirmationStatus {
    pub fn status(&self) -> bool {
        self.status
    }

    pub fn read_from(read: &mut dyn Read) -> Result<Self> {
        use crate::Error;
        use byteorder::ReadBytesExt;

        let status = read.read_u8().map_err(Error::Io)? == 1;

        Ok(ConfirmationStatus { status })
    }
    }
}
