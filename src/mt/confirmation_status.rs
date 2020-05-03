use crate::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// The Message Delivery Confirmation of a mobile-originated session.
///
/// The descriptions for these codes are taken directly from the `DirectIP` documentation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ConfirmationStatus {
    /// The Unique Client Message ID this message.
    pub message_id: u32,
    /// The device id.
    pub imei: [u8; 15],
    /// The Iridium Gateway id for this message.
    pub auto_id: u32,
    /// Message Status
    pub status: i16,
}
impl ConfirmationStatus {
    pub fn status(&self) -> bool {
        self.status > 0
    }

    pub fn read_from(read: &mut dyn Read) -> Result<Self> {
        use crate::Error;

        let message_id = read.read_u32::<BigEndian>().map_err(Error::Io)?;
        let mut imei = [0; 15];
        read.read_exact(&mut imei).map_err(Error::Io)?;
        let auto_id = read.read_u32::<BigEndian>().map_err(Error::Io)?;

        let status = read.read_i16::<BigEndian>().map_err(Error::Io)?;

        Ok(ConfirmationStatus {
            message_id,
            auto_id,
            imei,
            status,
        })
    }

    pub fn write_to<W: Write>(&self, write: &mut W) -> Result<()> {
        write.write_u8(0x44)?;
        write.write_u16::<BigEndian>(25)?;
        write.write_u32::<BigEndian>(self.message_id)?;
        write.write_all(&self.imei)?;
        write.write_u32::<BigEndian>(self.auto_id)?;
        write.write_i16::<BigEndian>(self.status)?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        28
    }
}
