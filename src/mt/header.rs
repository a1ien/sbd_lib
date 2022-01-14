use crate::information_element::SbdHeader;
use crate::{Imei, Result};
use std::io::{Read, Write};

/// A mobile-terminated header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    /// The Unique Client Message ID this message.
    pub message_id: u32,
    /// The device id.
    pub imei: Imei,
    /// The mobile-terminated Disposition Flags
    pub flags: u16,
}

impl SbdHeader for Header {
    fn write_to(&self, write: &mut dyn Write) -> Result<()> {
        use byteorder::{BigEndian, WriteBytesExt};

        write.write_u8(0x41)?;
        write.write_u16::<BigEndian>(21)?;
        write.write_u32::<BigEndian>(self.message_id)?;
        write.write_all(&self.imei)?;
        write.write_u16::<BigEndian>(self.flags)?;
        Ok(())
    }

    fn imei(&self) -> &str {
        use std::str;
        str::from_utf8(&self.imei).expect("IMEI numbers are specified to be ascii number")
    }
    fn len(&self) -> usize {
        24
    }
    fn as_mt(&self) -> Option<&Header> {
        Some(&self)
    }
}

impl Header {
    pub fn read_from(read: &mut dyn Read) -> Result<Header> {
        use crate::Error;
        use byteorder::{BigEndian, ReadBytesExt};
        let message_id = read.read_u32::<BigEndian>().map_err(Error::Io)?;
        let mut imei = [0; 15];
        read.read_exact(&mut imei).map_err(Error::Io)?;

        let flags = read.read_u16::<BigEndian>().map_err(Error::Io)?;

        Ok(Header {
            message_id,
            imei: imei.into(),
            flags,
        })
    }
}
