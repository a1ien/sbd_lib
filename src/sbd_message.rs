use crate::{
    information_element::{Header, InformationElement, SbdHeader},
    mo::LocationInformation,
    Result,
};
#[cfg(feature = "serde-derive")]
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    io::{Read, Write},
    path::Path,
};

const PROTOCOL_REVISION_NUMBER: u8 = 1;

/// A Iridium SBD message.

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde-derive", derive(Serialize, Deserialize))]
pub struct Message {
    header: Header,
    payload: Vec<u8>,
    location: Option<LocationInformation>,
    information_elements: Vec<InformationElement>,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Header: {}, payload: {:?}", self.header, self.payload)?;
        if let Some(location) = self.location {
            write!(f, ", location: {}", location)?;
        }
        if !self.information_elements.is_empty() {
            write!(f, ",ie {:?}", self.information_elements)?;
        }
        Ok(())
    }
}

impl Message {
    const PROTOCOL_REVISION_NUMBER_SIZE: usize = 1;
    const MESSAGE_SIZE_IN_HEADER: usize = 2;
    pub const HEADER_SIZE: usize =
        Self::PROTOCOL_REVISION_NUMBER_SIZE + Self::MESSAGE_SIZE_IN_HEADER;

    /// Returns this message's header.
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

    /// Create message from Path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        use std::fs::File;
        let file = File::open(path)?;
        Self::read_from(file)
    }

    /// Create message from Read
    pub fn read_from<R: Read>(mut read: R) -> Result<Self> {
        Self::create(InformationElement::parse(&mut read)?)
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

    /// Return overall message length with header
    pub fn length(&self) -> usize {
        self.header.len()
            + self.payload.len()
            + 1
            + 2
            + self.location.and_then(|l| Some(l.len())).unwrap_or(0)
            + self
                .information_elements
                .iter()
                .map(|ie| ie.len())
                .sum::<usize>()
            + Self::HEADER_SIZE
    }

    /// Creates a new message from information elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    ///         use sbd_lib::mo;
    ///         use sbd_lib::{Header, InformationElement, Message};
    ///         use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time};
    ///         let header = InformationElement::Header(
    ///             mo::Header {
    ///                 auto_id: 1,
    ///                 imei: [0; 15].into(),
    ///                 session_status: mo::SessionStatus::Ok,
    ///                 momsn: 1,
    ///                 mtmsn: 0,
    ///                 time_of_session: PrimitiveDateTime::new(
    ///                     Date::from_calendar_date(2017, Month::October, 1).unwrap(),
    ///                     Time::from_hms(0, 0, 0).unwrap(),
    ///                 )
    ///                 .assume_utc(),
    ///             }
    ///             .into(),
    ///         );
    ///         let payload = InformationElement::MOPayload(Vec::new());
    ///         let message = Message::create(vec![header, payload]);
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
                }
                InformationElement::MOPayload(p) | InformationElement::MTPayload(p) => {
                    if payload.is_some() {
                        return Err(Error::TwoPayloads);
                    } else {
                        payload = Some(p);
                    }
                }
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

        let payload = match self.header {
            Header::MOHeader(_) => InformationElement::MOPayload(self.payload.clone()),
            Header::MTHeader(_) => InformationElement::MTPayload(self.payload.clone()),
        };

        let overall_message_length = self.length() - 3;

        if overall_message_length > u16::MAX as usize {
            return Err(crate::Error::OverallMessageLength(overall_message_length));
        }

        write.write_u8(PROTOCOL_REVISION_NUMBER)?;
        write.write_u16::<BigEndian>(overall_message_length as u16)?;
        self.header.write_to(&mut write)?;
        payload.write_to(&mut write)?;
        self.location.and_then(|l| l.write_to(&mut write).ok());
        for information_element in &self.information_elements {
            information_element.write_to(&mut write)?;
        }
        Ok(())
    }

    /// Returns this message's location.
    pub fn location(&self) -> &Option<LocationInformation> {
        &self.location
    }

    /// Returns this message's information_elements.
    pub fn information_elements(&self) -> &[InformationElement] {
        &self.information_elements
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
    use crate::{mo, mt};
    use std::{fs::File, io::Cursor};
    use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time};

    fn mo_header() -> mo::Header {
        mo::Header {
            auto_id: 1,
            imei: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14].into(),
            session_status: mo::SessionStatus::Ok,
            momsn: 1,
            mtmsn: 0,
            time_of_session: PrimitiveDateTime::new(
                Date::from_calendar_date(2017, Month::October, 1).unwrap(),
                Time::from_hms(1, 2, 3).unwrap(),
            )
            .assume_utc(),
        }
    }

    fn location() -> mo::LocationInformation {
        mo::LocationInformation::new(0, (43, 30854), (41, 48860), Some(3))
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
        let message = Message::from(Header::from(mo::Header {
            auto_id: 1,
            imei: [
                0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x31, 0x32, 0x33, 0x34,
                0x35,
            ]
            .into(),
            session_status: mo::SessionStatus::Ok,
            momsn: 1,
            mtmsn: 0,
            time_of_session: OffsetDateTime::now_utc(),
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
        assert!(Message::create(vec![
            header.into(),
            InformationElement::MOPayload(vec![]),
            InformationElement::MOPayload(vec![])
        ])
        .is_err());
    }

    #[test]
    fn no_header() {
        assert!(Message::create(vec![InformationElement::MOPayload(vec![])]).is_err());
    }

    #[test]
    fn two_headers() {
        let header = mo_header();
        assert!(Message::create(vec![header.clone().into(), header.into()]).is_err());
    }

    #[test]
    fn two_location() {
        let location = location();
        assert!(Message::create(vec![location.clone().into(), location.into()]).is_err());
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
                PrimitiveDateTime::new(
                    Date::from_calendar_date(2015, Month::July, 9).unwrap(),
                    Time::from_hms(18, 15, 8).unwrap(),
                )
                .assume_utc(),
                header.time_of_session
            );
            assert_eq!(
                "test message from pete",
                String::from_utf8(message.payload().to_vec()).unwrap()
            );
        }
    }

    #[test]
    fn generate_save_and_read_back() {
        use std::io::Cursor;
        let header = Header::MOHeader(mo::Header {
            auto_id: 0x545645,
            imei: [0x31; 15].into(),
            session_status: mo::SessionStatus::Ok,
            momsn: 101,
            mtmsn: 102,
            time_of_session: OffsetDateTime::from_unix_timestamp(
                OffsetDateTime::now_utc().unix_timestamp(),
            )
            .unwrap(),
        });
        let mut buff = vec![];
        let message = Message::new(header, vec![], None, vec![]);
        message.write_to(&mut buff).unwrap();

        let read_back = Message::read_from(Cursor::new(buff)).unwrap();

        assert_eq!(message.header().as_mo(), read_back.header().as_mo())
    }

    #[test]
    fn sbd_mo_message_without_location() {
        let message = Message::from_path("data/0-mo.sbd").unwrap();

        let msg = Message::new(
            mo::Header {
                auto_id: 1894516585,
                session_status: mo::SessionStatus::Ok,
                imei: (*b"300234063904190").into(),
                momsn: 75,
                mtmsn: 0,
                time_of_session: OffsetDateTime::from_unix_timestamp(1436465708).unwrap(),
            }
            .into(),
            vec![
                116, 101, 115, 116, 32, 109, 101, 115, 115, 97, 103, 101, 32, 102, 114, 111, 109,
                32, 112, 101, 116, 101,
            ],
            None,
            vec![],
        );
        assert_eq!(msg, message);
    }

    #[test]
    fn sbd_mo_message_wit_location() {
        use std::fs::File;
        let file = File::open("data/1-mo-location.sbd").unwrap();
        let iei = InformationElement::parse(file).unwrap();

        let message = Message::create(iei).unwrap();
        let location = message.location().unwrap();

        let loc = mo::LocationInformation::new(0, (60, 5132), (29, 14176), Some(93));

        assert_eq!(loc, location);

        let mut buff = vec![];
        message.write_to(&mut buff).unwrap();
        let read_back = Message::read_from(Cursor::new(buff)).unwrap();
        assert_eq!(message, read_back);
    }

    #[test]
    fn sbd_responce_for_mt() {
        let file = File::open("data/resp.sbd").unwrap();
        let iei = InformationElement::parse(file).unwrap();
        let resp = InformationElement::Status(
            mt::ConfirmationStatus {
                message_id: 287454020,
                imei: (*b"300434060009290").into(),
                auto_id: 3064195606,
                status: 1,
            }
            .into(),
        );
        assert_eq!(resp, iei[0]);
    }

    #[test]
    fn sbd_mo_message_wit_location_big_data() {
        use std::fs::File;
        let file = File::open("data/data.sbd").unwrap();
        let iei = InformationElement::parse(file).unwrap();
        let message = Message::create(iei).unwrap();
        let msg = Message::new(
            mo::Header {
                auto_id: 26502124,
                session_status: mo::SessionStatus::Ok,
                imei: (*b"300434069104350").into(),
                momsn: 545,
                mtmsn: 0,
                time_of_session: OffsetDateTime::from_unix_timestamp(1587546484).unwrap(),
            }
            .into(),
            vec![
                10, 16, 18, 138, 173, 180, 242, 224, 34, 215, 130, 227, 62, 42, 16, 32, 165, 139,
                16, 1, 24, 129, 147, 128, 245, 5, 32, 129, 138, 188, 139, 1, 40, 8, 48, 75, 56,
                156, 167, 215, 186, 4, 64, 136, 224, 153, 172, 2, 72, 152, 1,
            ],
            Some(mo::LocationInformation::new(
                0,
                (59, 48741),
                (31, 30844),
                Some(3),
            )),
            vec![],
        );

        assert_eq!(msg, message);
    }
}
