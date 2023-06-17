pub use crate::{Error, Result};

/// Calculates the crc8 of the input data.
///
/// `CRC = X^8 + X^5 + X^4 + X^0`
pub fn calculate(data: &[u8]) -> u8 {
    let mut crc = 0;
    for byte in data {
        crc ^= byte;
        for _ in 0..u8::BITS {
            let bit = crc & 0x01;
            crc >>= 1;
            if bit != 0 {
                crc ^= 0x8C;
            }
        }
    }
    crc
}

/// Checks to see if data (including the crc byte) passes the crc check.
///
/// A nice property of this crc8 algorithm is that if you include the crc value
/// in the data it will always return 0, so it's not needed to separate the data
/// from the crc value
pub fn check(data: &[u8]) -> Result<()> {
    match calculate(data) {
        0 => Ok(()),
        crc8 => Err(Error::MismatchedCrc { crc8 }),
    }
}

#[test]
fn test() {
    assert_eq!(calculate(&[99, 1, 75, 70, 127, 255, 13, 16]), 21);
    assert_eq!(calculate(&[99, 1, 75, 70, 127, 255, 13, 16, 21]), 0);

    assert_eq!(calculate(&[97, 1, 75, 70, 127, 255, 15, 16]), 2);
    assert_eq!(calculate(&[97, 1, 75, 70, 127, 255, 15, 16, 2]), 0);

    assert_eq!(calculate(&[95, 1, 75, 70, 127, 255, 1, 16]), 155);
    assert_eq!(calculate(&[95, 1, 75, 70, 127, 255, 1, 16, 155]), 0);
}
