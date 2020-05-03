use crate::{
    information_element::{Header, InformationElement, SbdHeader},
    mo::LocationInformation,
    Result,
};
use std::{
    fmt,
    io::{Read, Write},
    path::Path,
};

const PROTOCOL_REVISION_NUMBER: u8 = 1;

/// A Iridium SBD message.

#[derive(Debug, Clone)]
pub struct Message {
    header: Header,
    payload: Vec<u8>,
    location: Option<LocationInformation>,
    information_elements: Vec<InformationElement>,
}

impl Message {
    pub fn header(&self) -> &dyn SbdHeader {
        &self.header
    }

    /// Returns this message's payload.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd_lib::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let payload = message.payload();
    /// ```
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        use std::fs::File;
        let file = File::open(path)?;
        Self::read_from(file)
    }

    pub fn read_from<R: Read>(mut read: R) -> Result<Self> {
        Self::create(InformationElement::read(&mut read)?)
    }

    pub fn new(
        header: Header,
        payload: Vec<u8>,
        location: Option<LocationInformation>,
        ie: Vec<InformationElement>,
    ) -> Self {
        Message {
            header,
            payload,
            location,
            information_elements: ie,
        }
    }

    /// Creates a new message from information elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// use chrono::{Utc, TimeZone};
    /// use sbd_lib::mo;
    /// use sbd_lib::{InformationElement, Header, Message};
    /// let header = InformationElement::Header(mo::Header {
    ///     auto_id: 1,
    ///     imei: [0; 15],
    ///     session_status: mo::SessionStatus::Ok,
    ///     momsn: 1,
    ///     mtmsn: 0,
    ///     time_of_session: Utc.ymd(2017, 10, 1).and_hms(0, 0, 0),
    /// }.into());
    /// let payload = InformationElement::Payload(Vec::new());
    /// let message = Message::create(vec![header, payload]);
    /// # }
    /// ```
    pub fn create<I: IntoIterator<Item = InformationElement>>(iter: I) -> Result<Self> {
        use crate::Error;

        let mut header: Option<Header> = None;
        let mut payload = None;
        let mut location = None;
        let mut information_elements = Vec::new();
        for information_element in iter {
            match information_element {
                InformationElement::Header(h) => {
                    if header.is_some() {
                        return Err(Error::TwoHeaders);
                    } else {
                        header = Some(h);
                    }
                },
                InformationElement::Payload(p) => if payload.is_some() {
                    return Err(Error::TwoPayloads);
                } else {
                    payload = Some(p);
                },
                InformationElement::LocationInformation(l) => {
                    if location.is_some() {
                        return Err(Error::TwoLocations);
                    } else {
                        location = Some(l);
                    }
                }
                ie => information_elements.push(ie),
            }
        }

        Ok(Self::new(
            header.ok_or(Error::NoHeader)?,
            payload.ok_or(Error::NoPayload)?,
            location,
            information_elements,
        ))
    }

    /// Returns this message's imei as a string.
    ///
    /// # Panics
    ///
    /// Panics if the IMEI number is not valid utf8. The specification says that IMEIs should be
    /// ascii numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd_lib::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let imei = message.imei();
    /// ```
    pub fn imei(&self) -> &str {
        self.header().imei()
    }

    /// Write this message back to a object that can `Write`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd_lib::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let mut buff = vec![];
    /// message.write_to(&mut buff);
    /// ```
    pub fn write_to<W: Write>(&self, mut write: W) -> Result<()> {
        use byteorder::{BigEndian, WriteBytesExt};

        let payload = InformationElement::from(self.payload.clone());
        let overall_message_length = self.header.len() + payload.len()
            + self.information_elements
                .iter()
                .map(|ie| ie.len())
                .sum::<usize>();
        if overall_message_length > u16::MAX as usize {
            return Err(Error::OverallMessageLength(overall_message_length));
        }

        write.write_u8(PROTOCOL_REVISION_NUMBER)?;
        write.write_u16::<BigEndian>(overall_message_length as u16)?;
        self.header.write_to(&mut write)?;
        payload.write_to(&mut write)?;
        for information_element in &self.information_elements {
            information_element.write_to(&mut write)?;
        }
        Ok(())
    }
}

impl From<Header> for Message {
    fn from(header: Header) -> Message {
        Message::new(header, vec![], None, vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mo;
    use chrono::{TimeZone, Utc};
    use std::fs::File;
    use std::str;

    pub fn mo_header() -> mo::Header {
        mo::Header {
            auto_id: 1,
            imei: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
            session_status: mo::SessionStatus::Ok,
            momsn: 1,
            mtmsn: 0,
            time_of_session: Utc.ymd(2017, 10, 1).and_hms(1, 2, 3),
        }
    }

    #[test]
    fn from_path() {
        Message::from_path("data/0-mo.sbd").unwrap();
    }

    #[test]
    fn from_read() {
        let file = File::open("data/0-mo.sbd").unwrap();
        Message::read_from(file).unwrap();
    }

    #[test]
    fn imei() {
        use chrono::Utc;
        let message = Message::from(Header::from(mo::Header {
            auto_id: 1,
            imei: [
                0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x31, 0x32, 0x33, 0x34,
                0x35,
            ],
            session_status: mo::SessionStatus::Ok,
            momsn: 1,
            mtmsn: 0,
            time_of_session: Utc::now(),
        }));
        assert_eq!("123456789012345", message.imei());
    }

    #[test]
    fn no_payload() {
        let header = mo_header();
        assert!(Message::create(vec![header.into()]).is_err());
    }

    #[test]
    fn two_payloads() {
        let header = mo_header();
        let payload = Vec::new();
        assert!(
            Message::create(vec![header.into(), payload.clone().into(), payload.into()]).is_err()
        );
    }

    #[test]
    fn no_header() {
        assert!(Message::create(vec![vec![].into()]).is_err());
    }

    #[test]
    fn two_headers() {
        let header = mo_header();
        assert!(Message::create(vec![header.clone().into(), header.into()]).is_err());
    }

    #[test]
    fn values() {
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        if let Some(header) = message.header().as_mo() {
            assert_eq!(1894516585, header.auto_id);
            assert_eq!("300234063904190", header.imei());
            assert_eq!(mo::SessionStatus::Ok, header.session_status);
            assert_eq!(75, header.momsn);
            assert_eq!(0, header.mtmsn);
            assert_eq!(
                Utc.ymd(2015, 7, 9).and_hms(18, 15, 8),
                header.time_of_session
            );
            assert_eq!(
                "test message from pete",
                str::from_utf8(message.payload()).unwrap()
            );
        }
    }

}
