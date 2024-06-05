use crate::{
    crc8::check,
    error::{Error, Result},
};

const CONVERSION_TIME: f32 = 750.0;

const NINE: u8 = 0b00011111;
const TEN: u8 = 0b00111111;
const ELEVEN: u8 = 0b01011111;
const TWELVE: u8 = 0b01111111;

/// Scratchpad
#[derive(Clone, Copy, Debug, Default)]
pub struct Scratchpad {
    pub temperature: f32,
    pub configuration: Configuration,
    pub triggers: Triggers,
    pub crc: u8,
}

impl TryFrom<[u8; 9]> for Scratchpad {
    type Error = Error;

    fn try_from(value: [u8; 9]) -> Result<Self> {
        check(&value)?;
        let configuration = Configuration::try_from(value[4])?;
        Ok(Scratchpad {
            temperature: temperature(value[0], value[1], configuration.resolution),
            triggers: Triggers {
                high: value[2] as _,
                low: value[3] as _,
            },
            configuration,
            crc: value[8],
        })
    }
}

/// Config
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Configuration {
    pub resolution: Resolution,
}

impl Configuration {
    /// Max conversion time (ms)
    pub fn conversion_time(&self) -> f32 {
        match self.resolution {
            Resolution::Nine => CONVERSION_TIME / 8.0,
            Resolution::Ten => CONVERSION_TIME / 4.0,
            Resolution::Eleven => CONVERSION_TIME / 2.0,
            Resolution::Twelve => CONVERSION_TIME,
        }
    }
}

impl TryFrom<u8> for Configuration {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            NINE => Ok(Self {
                resolution: Resolution::Nine,
            }),
            TEN => Ok(Self {
                resolution: Resolution::Ten,
            }),
            ELEVEN => Ok(Self {
                resolution: Resolution::Eleven,
            }),
            TWELVE => Ok(Self {
                resolution: Resolution::Twelve,
            }),
            _ => Err(Error::ConfigurationRegister),
        }
    }
}

impl From<Configuration> for u8 {
    fn from(value: Configuration) -> Self {
        match value.resolution {
            Resolution::Nine => NINE,
            Resolution::Ten => TEN,
            Resolution::Eleven => ELEVEN,
            Resolution::Twelve => TWELVE,
        }
    }
}

/// Temperature resolution: 9, 10, 11 or 12 bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Resolution {
    Nine,
    Ten,
    Eleven,
    #[default]
    Twelve,
}

/// Temperature triggers: high and low.
#[derive(Clone, Copy, Debug, Default)]
pub struct Triggers {
    pub high: i8,
    pub low: i8,
}

fn temperature(lsb: u8, msb: u8, resolution: Resolution) -> f32 {
    let divider = match resolution {
        Resolution::Nine => 2.0,
        Resolution::Ten => 4.0,
        Resolution::Eleven => 8.0,
        Resolution::Twelve => 16.0,
    };
    i16::from_be_bytes([lsb, msb]) as f32 / divider
}

#[test]
fn test() {
    // Configuration register
    assert_eq!(
        Err(Error::ConfigurationRegister),
        Configuration::try_from(0b0_11_11110),
    );
    assert_eq!(
        Err(Error::ConfigurationRegister),
        Configuration::try_from(0b0_11_11101),
    );
    assert_eq!(
        Err(Error::ConfigurationRegister),
        Configuration::try_from(0b0_11_11011),
    );
    assert_eq!(
        Err(Error::ConfigurationRegister),
        Configuration::try_from(0b0_11_10111),
    );
    assert_eq!(
        Err(Error::ConfigurationRegister),
        Configuration::try_from(0b0_11_01111),
    );
    assert_eq!(
        Err(Error::ConfigurationRegister),
        Configuration::try_from(0b1_11_11111),
    );
    // Temperature
    assert_eq!(125.0, temperature(0x07, 0xD0, Default::default()));
    assert_eq!(85.0, temperature(0x05, 0x50, Default::default()));
    assert_eq!(25.0625, temperature(0x01, 0x91, Default::default()));
    assert_eq!(10.125, temperature(0x00, 0xA2, Default::default()));
    assert_eq!(0.5, temperature(0x00, 0x08, Default::default()));
    assert_eq!(0.0, temperature(0x00, 0x00, Default::default()));
    assert_eq!(-0.5, temperature(0xFF, 0xF8, Default::default()));
    assert_eq!(-10.125, temperature(0xFF, 0x5E, Default::default()));
    assert_eq!(-25.0625, temperature(0xFE, 0x6F, Default::default()));
    assert_eq!(-55.0, temperature(0xFC, 0x90, Default::default()));
}
