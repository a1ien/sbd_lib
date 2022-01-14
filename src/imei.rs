#[cfg(feature = "serde-derive")]
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-derive", derive(Serialize, Deserialize))]
pub struct Imei(
    #[cfg_attr(
        feature = "serde-derive",
        serde(serialize_with = "as_str", deserialize_with = "str_to_imei")
    )]
    [u8; 15],
);

impl Deref for Imei {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl DerefMut for Imei {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

impl From<[u8; 15]> for Imei {
    fn from(val: [u8; 15]) -> Self {
        Self(val)
    }
}

impl PartialEq<Imei> for [u8; 15] {
    fn eq(&self, other: &Imei) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<[u8; 15]> for Imei {
    fn eq(&self, other: &[u8; 15]) -> bool {
        self.0.eq(other)
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
fn str_to_imei<'de, D>(deserializer: D) -> std::result::Result<[u8; 15], D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;
    let mut value = [0; 15];
    let data = <&'de [u8]>::deserialize(deserializer)?;

    if data.len() == value.len() && data.iter().all(u8::is_ascii_digit) {
        value.copy_from_slice(data);
        Ok(value)
    } else {
        Err(de::Error::custom("wrong imei"))
    }
}
