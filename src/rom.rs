use crate::crc8::{check, Error, Result};
use core::fmt::Debug;

/// Lasered ROM
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Rom {
    pub family_code: u8,
    pub serial_number: [u8; 6],
    pub crc: u8,
}

impl TryFrom<[u8; 8]> for Rom {
    type Error = Error;

    fn try_from(value: [u8; 8]) -> Result<Self> {
        check(&value)?;
        Ok(Self {
            family_code: value[0],
            serial_number: [value[1], value[2], value[3], value[4], value[5], value[6]],
            crc: value[7],
        })
    }
}

impl TryFrom<u64> for Rom {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self> {
        value.to_le_bytes().try_into()
    }
}

impl From<Rom> for [u8; 8] {
    fn from(value: Rom) -> Self {
        [
            value.family_code,
            value.serial_number[0],
            value.serial_number[1],
            value.serial_number[2],
            value.serial_number[3],
            value.serial_number[4],
            value.serial_number[5],
            value.crc,
        ]
    }
}

impl From<Rom> for u64 {
    fn from(value: Rom) -> Self {
        u64::from_le_bytes(value.into())
    }
}

#[test]
fn test() {
    assert_eq!(
        Ok(Rom {
            family_code: 0x28,
            serial_number: [0x00; 6],
            crc: 0x1E,
        }),
        Rom::try_from(0x1E_000000000000_28)
    );
    assert_eq!(
        Ok(Rom {
            family_code: 0x28,
            serial_number: [0xFF; 6],
            crc: 0xC,
        }),
        Rom::try_from(0x0C_FFFFFFFFFFFF_28)
    );
}
