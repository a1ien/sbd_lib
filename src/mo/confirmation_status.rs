use crate::Result;
use byteorder::WriteBytesExt;
#[cfg(feature = "serde-derive")]
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// The Message Delivery Confirmation of a mobile-originated session.
///
/// The descriptions for these codes are taken directly from the `DirectIP` documentation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde-derive", derive(Serialize, Deserialize))]
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

    pub fn write_to<W: Write>(&self, write: &mut W) -> Result<()> {
        write.write_u8(self.status as u8)?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        4
    }
}
