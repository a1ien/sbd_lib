use crate::information_element::SbdHeader;
use crate::mo::session_status::SessionStatus;
use crate::Result;
use chrono::{DateTime, Utc};
use std::io::{Read, Write};

/// A mobile-originated header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    /// The Iridium Gateway id for this message.
    pub auto_id: u32,
    /// The device id.
    pub imei: [u8; 15],
    /// The session status.
    pub session_status: SessionStatus,
    /// The mobile originated message sequence number.
    pub momsn: u16,
    /// The mobile terminated message sequence number.
    pub mtmsn: u16,
    /// The time of iridium session.
    pub time_of_session: DateTime<Utc>,
}

impl SbdHeader for Header {
    fn write_to(&self, write: &mut dyn Write) -> Result<()> {
        use crate::Error;
        use byteorder::{BigEndian, WriteBytesExt};

        write.write_u8(1)?;
        write.write_u16::<BigEndian>(31)?;
        write.write_u32::<BigEndian>(self.auto_id)?;
        write.write_all(&self.imei)?;
        write.write_u8(self.session_status as u8)?;
        write.write_u16::<BigEndian>(self.momsn)?;
        write.write_u16::<BigEndian>(self.mtmsn)?;
        let timestamp = self.time_of_session.timestamp();
        if timestamp < 0 {
            return Err(Error::NegativeTimestamp(timestamp));
        } else {
            write.write_u32::<BigEndian>(timestamp as u32)?;
        };
        Ok(())
    }

    fn imei(&self) -> &str {
        use std::str;
        str::from_utf8(&self.imei).expect("IMEI numbers are specified to be ascii number")
    }
    fn len(&self) -> usize {
        31
    }
    fn as_mo(&self) -> Option<&Header> {
        Some(&self)
    }
}

impl Header {
    pub fn read_from(read: &mut dyn Read) -> Result<Self> {
        use crate::Error;
        use byteorder::{BigEndian, ReadBytesExt};
        use chrono::TimeZone;

        let auto_id = read.read_u32::<BigEndian>().map_err(Error::Io)?;
        let mut imei = [0; 15];
        read.read_exact(&mut imei).map_err(Error::Io)?;
        let session_status = SessionStatus::new(read.read_u8().map_err(Error::Io)?)?;
        let momsn = read.read_u16::<BigEndian>().map_err(Error::Io)?;
        let mtmsn = read.read_u16::<BigEndian>().map_err(Error::Io)?;
        let time_of_session = read
            .read_u32::<BigEndian>()
            .map_err(Error::from)
            .map(|n| Utc.timestamp(i64::from(n), 0))?;
        Ok(Header {
            auto_id,
            imei,
            session_status,
            momsn,
            mtmsn,
            time_of_session,
        })
    }
    /// Returns this message's session status.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd_lib::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// if let Some(header) =  message.header().as_mo() {
    ///     let session_status = header.session_status();
    /// }
    /// ```
    pub fn session_status(&self) -> SessionStatus {
        self.session_status
    }

    /// Returns this message's mobile originated message sequence number.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd_lib::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// if let Some(header) =  message.header().as_mo() {
    ///     let momsn = header.momsn();
    /// }
    /// ```
    pub fn momsn(&self) -> u16 {
        self.momsn
    }

    /// Returns this message's mobile terminated message sequence number.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd_lib::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// if let Some(header) = message.header().as_mo() {
    ///     let mtmsn = header.mtmsn();
    /// }
    /// ```
    pub fn mtmsn(&self) -> u16 {
        self.mtmsn
    }

    /// Returns this message's time of session.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd_lib::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// if let Some(header) = message.header().as_mo() {
    ///     let time_of_session = header.time_of_session();
    /// }
    /// ```
    pub fn time_of_session(&self) -> DateTime<Utc> {
        self.time_of_session
    }
}
