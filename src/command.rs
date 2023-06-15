use crate::OneWire;
use embedded_hal::{
    delay::DelayUs,
    digital::{ErrorType, InputPin, OutputPin},
};

/// Commander
pub trait Commander<T> {
    fn run<C: Command<T>>(&mut self, command: C) -> C::Output;
}

impl<T, U: DelayUs> Commander<T> for OneWire<T, U> {
    fn run<C: Command<T>>(&mut self, command: C) -> C::Output {
        command.execute(self)
    }
}

/// Command
pub trait Command<T> {
    type Output;

    fn execute(&self, one_wire: &mut OneWire<T, impl DelayUs>) -> Self::Output;
}

/// Read byte command
///
/// Read 1-Wire data byte.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReadByte;

impl<T: InputPin + OutputPin + ErrorType> Command<T> for ReadByte {
    type Output = Result<u8, T::Error>;

    fn execute(&self, one_wire: &mut OneWire<T, impl DelayUs>) -> Self::Output {
        let mut byte = 0;
        for _ in 0..u8::BITS {
            byte >>= 1;
            if one_wire.read_bit()? {
                byte |= 0x80;
            }
        }
        Ok(byte)
    }
}

/// Read bytes command
///
/// Read 1-Wire data bytes.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReadBytes<const N: usize>;

impl<const N: usize, T: InputPin + OutputPin + ErrorType> Command<T> for ReadBytes<N> {
    type Output = Result<[u8; N], T::Error>;

    fn execute(&self, one_wire: &mut OneWire<T, impl DelayUs>) -> Self::Output {
        let mut bytes = [0; N];
        for byte in &mut bytes {
            *byte = one_wire.run(ReadByte)?;
        }
        Ok(bytes)
    }
}

/// Reset command
pub struct Reset;

impl<T: InputPin + OutputPin + ErrorType> Command<T> for Reset {
    type Output = Result<bool, T::Error>;

    fn execute(&self, one_wire: &mut OneWire<T, impl DelayUs>) -> Self::Output {
        one_wire.reset()
    }
}

/// Write byte command
///
/// Write 1-Wire data byte.
pub struct WriteByte {
    pub byte: u8,
}

impl<T: InputPin + OutputPin + ErrorType> Command<T> for WriteByte {
    type Output = Result<(), T::Error>;

    fn execute(&self, one_wire: &mut OneWire<T, impl DelayUs>) -> Self::Output {
        let mut byte = self.byte;
        for _ in 0..u8::BITS {
            one_wire.write_bit(byte & 0x01 == 0x01)?;
            byte >>= 1;
        }
        Ok(())
    }
}

/// Write bytes command
///
/// Write 1-Wire data bytes.
pub struct WriteBytes {
    pub bytes: [u8],
}

impl<T: InputPin + OutputPin + ErrorType> Command<T> for WriteBytes {
    type Output = Result<(), T::Error>;

    fn execute(&self, one_wire: &mut OneWire<T, impl DelayUs>) -> Self::Output {
        for &byte in &self.bytes {
            one_wire.run(WriteByte { byte })?;
        }
        Ok(())
    }
}

/// Wait command
pub struct Wait {
    pub us: u32,
}

impl<T> Command<T> for Wait {
    type Output = ();

    fn execute(&self, one_wire: &mut OneWire<T, impl DelayUs>) -> Self::Output {
        one_wire.delay.delay_us(self.us);
    }
}
