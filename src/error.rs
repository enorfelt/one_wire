use core::convert::Infallible;
use thiserror::Error;

/// Result
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Error
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum Error {
    #[error("invalid configuration register (resolution)")]
    ConfigurationRegister,
    #[error("the bus was expected to be pulled high by a ~5K ohm pull-up resistor, but it wasn't")]
    NotHigh,
    #[error("pin")]
    Pin(#[from] Infallible),
    #[error("family code mismatch")]
    MismatchedFamilyCode,
    #[error("there are no devices attached to the 1-Wire bus")]
    NoAttachedDevices,
    #[error("CRC mismatch {{ crc8={crc8} }}")]
    MismatchedCrc { crc8: u8 },
    #[error("timeout expired")]
    Timeout,
}
