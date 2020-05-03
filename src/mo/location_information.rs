use crate::Result;
use std::{fmt, io::Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LocationDirection {
    NE,
    NW,
    SE,
    SW,
}

impl From<u8> for LocationDirection {
    fn from(flag: u8) -> Self {
        match flag {
            0x40 => LocationDirection::SE,
            0x80 => LocationDirection::NW,
            0xC0 => LocationDirection::SW,
            _ => LocationDirection::NE,
        }
    }
}

impl From<LocationDirection> for u8 {
    fn from(direction: LocationDirection) -> Self {
        match direction {
            LocationDirection::NE => 0,
            LocationDirection::SE => 0x40,
            LocationDirection::NW => 0x80,
            LocationDirection::SW => 0xC0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationInformation {
    direction: LocationDirection,
    latitude: (u8, u16),
    longitude: (u8, u16),
    radius: Option<u32>,
}

impl fmt::Display for LocationInformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let latitude =
            i32::from(self.latitude.0) * 10_000_000 + (i32::from(self.latitude.1) * 10000) / 60;
        let longitude =
            i32::from(self.longitude.0) * 10_000_000 + (i32::from(self.longitude.1) * 10000) / 60;
        write!(
            f,
            "direction: {:?}, latitude: {:.7}, longitude: {:.7} radius: {} km",
            self.direction,
            f64::from(latitude) * 1e-7,
            f64::from(longitude) * 1e-7,
            self.radius.unwrap_or(0xffff)
        )
    }
}

impl LocationInformation {
    pub fn new(
        flags: u8,
        latitude: (u8, u16),
        longitude: (u8, u16),
        radius: Option<u32>,
    ) -> LocationInformation {
        let direction = flags.into();
        Self {
            direction,
            latitude,
            longitude,
            radius,
        }
    }

    pub fn latitude(&self) {}

    pub fn longitude(&self) {}

    pub fn len(&self) -> usize {
        match self.radius {
            None => 10,
            Some(_) => 14,
        }
    }

    pub fn write_to<W: Write>(&self, write: &mut W) -> Result<()> {
        use byteorder::{BigEndian, WriteBytesExt};
        // let flag: u8 =
        //     if self.latitude > 0 { 0 } else { 1 } | if self.longitude > 0 { 0 } else { 1 << 1 };
        write.write_u8(3)?;
        write.write_u16::<BigEndian>((self.len() - 3) as u16)?;
        write.write_u8(self.direction.into())?;

        write.write_u8(self.latitude.0)?;

        write.write_u16::<BigEndian>(self.latitude.1)?;

        write.write_u8(self.longitude.0)?;

        write.write_u16::<BigEndian>(self.longitude.1)?;
        match self.radius {
            Some(radius) => write.write_u32::<BigEndian>(radius)?,
            _ => {}
        };

        Ok(())
    }
}
