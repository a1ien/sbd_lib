use crate::information_element::SbdHeader;
use crate::mo::session_status::SessionStatus;
use crate::Result;
#[cfg(feature = "serde-derive")]
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
#[cfg(feature = "serde-derive")]
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// A mobile-originated header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-derive", derive(Serialize, Deserialize))]
pub struct Header {
    /// The Iridium Gateway id for this message.
    pub auto_id: u32,
    /// The device id.
    #[cfg_attr(
        feature = "serde-derive",
        serde(serialize_with = "as_str", deserialize_with = "str_to_imei")
    )]
    pub imei: [u8; 15],
    /// The session status.
    pub session_status: SessionStatus,
    /// The mobile originated message sequence number.
    pub momsn: u16,
    /// The mobile terminated message sequence number.
    pub mtmsn: u16,
    /// The time of iridium session.
    #[cfg_attr(
        feature = "serde-derive",
        serde(serialize_with = "as_rfc3339", deserialize_with = "from_rfc3339")
    )]
    pub time_of_session: OffsetDateTime,
}

impl SbdHeader for Header {
    fn write_to(&self, write: &mut dyn Write) -> Result<()> {
        use crate::Error;
        use byteorder::{BigEndian, WriteBytesExt};

        write.write_u8(1)?;
        write.write_u16::<BigEndian>(28)?;
        write.write_u32::<BigEndian>(self.auto_id)?;
        write.write_all(&self.imei)?;
        write.write_u8(self.session_status as u8)?;
        write.write_u16::<BigEndian>(self.momsn)?;
        write.write_u16::<BigEndian>(self.mtmsn)?;
        let timestamp = self.time_of_session.unix_timestamp();
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

        let auto_id = read.read_u32::<BigEndian>().map_err(Error::Io)?;
        let mut imei = [0; 15];
        read.read_exact(&mut imei).map_err(Error::Io)?;
        let session_status = SessionStatus::new(read.read_u8().map_err(Error::Io)?)?;
        let momsn = read.read_u16::<BigEndian>().map_err(Error::Io)?;
        let mtmsn = read.read_u16::<BigEndian>().map_err(Error::Io)?;
        let time_of_session = read.read_u32::<BigEndian>().map_err(Error::from).map(|n| {
            OffsetDateTime::from_unix_timestamp(i64::from(n)).expect("We convert from u32 time")
        })?;
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
    pub fn time_of_session(&self) -> OffsetDateTime {
        self.time_of_session
    }
}

#[cfg(feature = "serde-derive")]
fn as_str<S>(imei: &[u8], serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser;
    use std::str;
    match str::from_utf8(imei) {
        Ok(s) => serializer.serialize_str(s),
        _ => Err(ser::Error::custom("imei contains invalid UTF-8 characters")),
    }
}

#[cfg(feature = "serde-derive")]
pub fn str_to_imei<'de, D>(deserializer: D) -> std::result::Result<[u8; 15], D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;
    let mut value = [0; 15];
    let data = <&'de [u8]>::deserialize(deserializer)?;
    if data.len() == value.len() {
        value.copy_from_slice(data);
        Ok(value)
    } else {
        Err(de::Error::custom("imei wrong len"))
    }
}

#[cfg(feature = "serde-derive")]
fn as_rfc3339<S>(
    time_of_session: &OffsetDateTime,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser;
    match time_of_session.format(&Rfc3339) {
        Ok(s) => serializer.serialize_str(&s),
        _ => Err(ser::Error::custom("time_of_session contains data")),
    }
}

#[cfg(feature = "serde-derive")]
pub fn from_rfc3339<'de, D>(deserializer: D) -> std::result::Result<OffsetDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    let data = <&'de str>::deserialize(deserializer)?;
    OffsetDateTime::parse(data, &Rfc3339)
        .map_err(|_| de::Error::custom("time_of_session contains data"))
}
