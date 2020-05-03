use std::fmt::Debug;
use Result;
use mo;
use mt;
use std::io::{Read, Write};

const PROTOCOL_REVISION_NUMBER: u8 = 1;

pub trait SbdHeader: Debug {
    //fn read_from(read: &Read) -> Result<Box<Self>>;
    fn write_to(&self, write: &mut Write) -> Result<()>;
    fn imei(&self) -> &str;
    fn len(&self) -> usize;
    fn as_mo(&self) -> Option<&mo::Header> {
        None
    }
    fn as_mt(&self) -> Option<&mt::Header> {
        None
    }
}

/// A mobile-originated or mobile-terminated header
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Header {
    /// Information element holding the mobile-originated header.
    MOHeader(mo::Header),
    /// Information element holding the mobile-terminated header.
    MTHeader(mt::Header),
}

impl SbdHeader for Header {
    fn write_to(&self, write: &mut Write) -> Result<()> {
        match *self {
            Header::MOHeader(ref header) => header.write_to(write)?,
            Header::MTHeader(ref header) => header.write_to(write)?,
        }
        Ok(())
    }

    fn imei(&self) -> &str {
        match *self {
            Header::MOHeader(ref header) => header.imei(),
            Header::MTHeader(ref header) => header.imei(),
        }
    }

    fn len(&self) -> usize {
        match *self {
            Header::MOHeader(ref header) => header.len(),
            Header::MTHeader(ref header) => header.len(),
        }
    }
    fn as_mo(&self) -> Option<&mo::Header> {
        if let Header::MOHeader(ref header) = *self {
            Some(header)
        } else {
            None
        }
    }

    fn as_mt(&self) -> Option<&mt::Header> {
        if let Header::MTHeader(ref header) = *self {
            Some(header)
        } else {
            None
        }
    }
}

impl From<mo::Header> for Header {
    fn from(header: mo::Header) -> Self {
        Header::MOHeader(header)
    }
}

impl From<mt::Header> for Header {
    fn from(header: mt::Header) -> Self {
        Header::MTHeader(header)
    }
}

/// A mobile-originated or mobile-terminated status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Information element holding the mobile-originated status.
    MOStatus(mo::ConfirmationStatus),
    /// Information element holding the mobile-terminated status.
    MTStatus(mt::ConfirmationStatus),
}

impl From<mo::ConfirmationStatus> for Status {
    fn from(status: mo::ConfirmationStatus) -> Self {
        Status::MOStatus(status)
    }
}

impl From<mt::ConfirmationStatus> for Status {
    fn from(status: mt::ConfirmationStatus) -> Self {
        Status::MTStatus(status)
    }
}

/// A information element, or IE.
///
/// These are the building blocks of a SBD message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InformationElement {
    /// Information element holding the header MO or MT.
    Header(Header),
    /// The mobile originated payload.
    Payload(Vec<u8>),
    /// Message Delivery Confirmation
    Status(Status),
    /// The mobile originated location information.
    LocationInformation(LocationInformation),
}

impl InformationElement {
    /// Reads this information element from a `Read`.
    pub fn read_from<R: Read>(mut read: R) -> Result<Self> {
        use Error;
        use byteorder::{BigEndian, ReadBytesExt};

        let iei = read.read_u8().map_err(Error::Io)?;
        let length = read.read_u16::<BigEndian>().map_err(Error::Io)?;
        match iei {
            0x1 => Ok(mo::Header::read_from(&mut read)?.into()),
            0x41 => Ok(mt::Header::read_from(&mut read)?.into()),
            0x2 | 0x42 => {
                let mut payload = vec![0; length as usize];
                read.read_exact(&mut payload).map_err(Error::Io)?;
                Ok(InformationElement::Payload(payload))
            }
            0x3 => {
                let mut location = vec![0; length as usize];
                read.read_exact(&mut location).map_err(Error::Io)?;
                let mut cur = Cursor::new(&location);
                let flags = cur.read_u8()?;
                let latitude = (cur.read_u8()?, cur.read_u16::<BigEndian>()?);
                let longitude = (cur.read_u8()?, cur.read_u16::<BigEndian>()?);

                let radius = if length == 11 {
                    Some(cur.read_u32::<BigEndian>()?)
                } else {
                    None
                };

                let loc = LocationInformation::new(flags, latitude, longitude, radius);

                Ok(InformationElement::LocationInformation(loc))
            }
            0x44 => Ok(mt::ConfirmationStatus::read_from(&mut read)?.into()),
            0x5 => Ok(mo::ConfirmationStatus::read_from(&mut read)?.into()),

            _ => Err(Error::InvalidInformationElementIdentifier(iei)),
        }
    }

    pub fn read<R: Read>(mut read: R) -> Result<Vec<Self>> {
        use Error;
        use byteorder::{BigEndian, ReadBytesExt};
        use std::io::Cursor;

        let protocol_revision_number = read.read_u8()?;
        if protocol_revision_number != PROTOCOL_REVISION_NUMBER {
            return Err(Error::InvalidProtocolRevisionNumber(
                protocol_revision_number,
            ));
        }
        let overall_message_length = read.read_u16::<BigEndian>()?;
        let mut message = vec![0; overall_message_length as usize];
        read.read_exact(&mut message)?;

        let mut cursor = Cursor::new(message);
        let mut information_elements = Vec::new();
        while cursor.position() < u64::from(overall_message_length) {
            information_elements.push(InformationElement::read_from(&mut cursor)?);
        }
        Ok(information_elements)
    }

    /// Returns the length of this information element, including the information element header.
    pub fn len(&self) -> usize {
        match *self {
            InformationElement::Header(ref h) => h.len(),
            InformationElement::Status(Status::MOStatus(_)) => 1,
            InformationElement::Status(Status::MTStatus(_)) => 25,
            InformationElement::Payload(ref payload) => 3 + payload.len(),
            InformationElement::LocationInformation(location) => location.len(),
        }
    }

    /// Returns true if this information element is empty.
    ///
    /// At this point, only can be true if the payload is empty.
    pub fn is_empty(&self) -> bool {
        match *self {
            InformationElement::Payload(ref payload) => payload.is_empty(),
            _ => false,
        }
    }

    /// Writes this information element to a `Write`.
    pub fn write_to<W: Write>(&self, mut write: &mut W) -> Result<()> {
        use byteorder::{BigEndian, WriteBytesExt};
        use Error;
        use std::u16;

        match *self {
            InformationElement::Header(ref header) => {
                header.write_to(&mut write)?;
            }
            InformationElement::Status(_) => {
                unimplemented!();
            }
            InformationElement::Payload(ref payload) => {
                write.write_u8(2)?;
                let len = payload.len();
                if len > u16::MAX as usize {
                    return Err(Error::PayloadTooLong(len));
                } else {
                    write.write_u16::<BigEndian>(len as u16)?;
                }
                write.write_all(payload)?;
            }
            InformationElement::LocationInformation(location) => {
                location.write_to(&mut write)?;
            }
        }
        Ok(())
    }
}

impl From<mo::ConfirmationStatus> for InformationElement {
    fn from(status: mo::ConfirmationStatus) -> Self {
        InformationElement::Status(Status::from(status))
    }
}

impl From<mt::ConfirmationStatus> for InformationElement {
    fn from(status: mt::ConfirmationStatus) -> Self {
        InformationElement::Status(Status::from(status))
    }
}

impl From<mo::Header> for InformationElement {
    fn from(header: mo::Header) -> Self {
        InformationElement::Header(Header::from(header))
    }
}

impl From<mt::Header> for InformationElement {
    fn from(header: mt::Header) -> Self {
        InformationElement::Header(Header::from(header))
    }
}

impl From<Vec<u8>> for InformationElement {
    fn from(payload: Vec<u8>) -> InformationElement {
        InformationElement::Payload(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::fs::File;
    use std::io::{Cursor, Read, Seek, SeekFrom};

    #[test]
    fn read_from() {
        let mut file = File::open("data/0-mo.sbd").unwrap();
        file.seek(SeekFrom::Start(3)).unwrap();
        {
            let read = Read::by_ref(&mut file).take(31);
            match InformationElement::read_from(read).unwrap() {
                InformationElement::Header(header) => {
                    if let Some(header) = header.as_mo() {
                        assert_eq!(1894516585, header.auto_id);
                        assert_eq!(b"300234063904190", &header.imei);
                        assert_eq!(mo::SessionStatus::Ok, header.session_status);
                        assert_eq!(75, header.momsn);
                        assert_eq!(0, header.mtmsn);
                        assert_eq!(
                            Utc.ymd(2015, 7, 9).and_hms(18, 15, 08),
                            header.time_of_session
                        );
                    } else {
                        panic!("Unexpected information element")
                    }
                }
                _ => panic!("Unexpected information element"),
            }
        }
        match InformationElement::read_from(file).unwrap() {
            InformationElement::Payload(data) => {
                assert_eq!(b"test message from pete", data.as_slice())
            }
            _ => panic!("Unexpected information element"),
        }
    }

    #[test]
    fn undersized() {
        let mut file = File::open("data/0-mo.sbd").unwrap();
        file.seek(SeekFrom::Start(3)).unwrap();
        let read = file.take(30);
        assert!(InformationElement::read_from(read).is_err());
    }

    #[test]
    fn header_len() {
        let header = mo::Header {
            auto_id: 1,
            imei: [0; 15],
            session_status: mo::SessionStatus::Ok,
            momsn: 1,
            mtmsn: 1,
            time_of_session: Utc.ymd(2017, 10, 17).and_hms(12, 0, 0),
        };
        let ie = InformationElement::Header(header.into());
        assert_eq!(31, ie.len());
    }

    #[test]
    fn payload_len() {
        assert_eq!(4, InformationElement::from(vec![1]).len());
    }

    #[test]
    fn location_information_len() {
        assert_eq!(
            10,
            InformationElement::LocationInformation(LocationInformation::new(
                0,
                (60, 1111),
                (60, 1111),
                None
            ))
            .len()
        );

        assert_eq!(
            14,
            InformationElement::LocationInformation(LocationInformation::new(
                0,
                (60, 1111),
                (60, 1111),
                Some(5)
            ))
            .len()
        );
    }

    #[test]
    fn roundtrip_header() {
        let header = mo::Header {
            auto_id: 1,
            imei: [0; 15],
            session_status: mo::SessionStatus::Ok,
            momsn: 1,
            mtmsn: 1,
            time_of_session: Utc.ymd(2017, 10, 17).and_hms(12, 0, 0),
        };
        let ie = InformationElement::Header(header.into());
        let mut cursor = Cursor::new(Vec::new());
        ie.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        assert_eq!(ie, InformationElement::read_from(&mut cursor).unwrap());
    }

    #[test]
    fn header_time_of_session_too_old() {
        let header = mo::Header {
            auto_id: 1,
            imei: [0; 15],
            session_status: mo::SessionStatus::Ok,
            momsn: 1,
            mtmsn: 1,
            time_of_session: Utc.ymd(1969, 12, 31).and_hms(23, 59, 59),
        };
        assert!(
            InformationElement::Header(header.into())
                .write_to(&mut Cursor::new(Vec::new()))
                .is_err()
        );
    }

    #[test]
    fn roundtrip_payload() {
        let payload = vec![1];
        let ie = InformationElement::from(payload);
        let mut cursor = Cursor::new(Vec::new());
        ie.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        assert_eq!(ie, InformationElement::read_from(cursor).unwrap());
    }

    #[test]
    fn payload_too_long() {
        use std::u16;
        let payload = vec![0; u16::MAX as usize + 1];
        assert!(
            InformationElement::from(payload)
                .write_to(&mut Cursor::new(Vec::new()))
                .is_err()
        );
    }

    #[test]
    fn roundtrip_location_information() {
        let ie = InformationElement::LocationInformation(LocationInformation::new(
            0,
            (60, 1111),
            (60, 1111),
            None,
        ));
        let mut cursor = Cursor::new(Vec::new());
        ie.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        assert_eq!(ie, ie);
    }
}
