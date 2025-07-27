use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MacAddress([u8; 6]);

#[derive(Debug)]
pub enum MacAddressParseError {
    InvalidDigit,
    InvalidLength,
}

impl Display for MacAddressParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MacAddressParseError::InvalidDigit => write!(f, "Invalid digit"),
            MacAddressParseError::InvalidLength => write!(f, "Invalid length"),
        }
    }
}

impl FromStr for MacAddress {
    type Err = MacAddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s
            .split(':')
            .map(|part| u8::from_str_radix(part, 16))
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|_| MacAddressParseError::InvalidDigit)?;

        if bytes.len() != 6 {
            Err(MacAddressParseError::InvalidLength)
        } else {
            Ok(MacAddress(bytes.try_into().unwrap()))
        }
    }
}

impl Serialize for MacAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        ))
    }
}

impl<'de> Deserialize<'de> for MacAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}
