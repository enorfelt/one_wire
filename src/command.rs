use crate::OneWire;
use core::convert::Infallible;
use embedded_hal::{
    delay::DelayUs,
    digital::{ErrorType, InputPin, OutputPin},
};

/// Commander
pub trait Commander {
    fn run<C: Command>(&mut self, command: C) -> C::Output;
}

impl<T: InputPin + OutputPin + ErrorType<Error = Infallible>, U: DelayUs> Commander
    for OneWire<T, U>
{
    fn run<C: Command>(&mut self, command: C) -> C::Output {
        command.execute(self)
    }
}

/// Command
pub trait Command {
    type Output;

    fn execute(
        &self,
        one_wire: &mut OneWire<
            impl InputPin + OutputPin + ErrorType<Error = Infallible>,
            impl DelayUs,
        >,
    ) -> Self::Output;
}
