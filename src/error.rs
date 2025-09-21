use core::convert::Infallible;
use core::fmt;

/// Result
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Error
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    ConfigurationRegister,
    NotHigh,
    Pin(Infallible),
    MismatchedFamilyCode,
    NoAttachedDevices,
    MismatchedCrc { crc8: u8 },
    Timeout,
    UnexpectedResponse,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConfigurationRegister => write!(f, "invalid configuration register (resolution)"),
            Error::NotHigh => write!(f, "the bus was expected to be pulled high by a ~5K ohm pull-up resistor, but it wasn't"),
            Error::Pin(_) => write!(f, "pin error"),
            Error::MismatchedFamilyCode => write!(f, "family code mismatch"),
            Error::NoAttachedDevices => write!(f, "there are no devices attached to the 1-Wire bus"),
            Error::MismatchedCrc { crc8 } => write!(f, "CRC mismatch {{ crc8={} }}", crc8),
            Error::Timeout => write!(f, "timeout expired"),
            Error::UnexpectedResponse => write!(f, "unexpected response"),
        }
    }
}

impl From<Infallible> for Error {
    fn from(never: Infallible) -> Self {
        Error::Pin(never)
    }
}
