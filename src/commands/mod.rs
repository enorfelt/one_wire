pub use self::{
    memory::{
        MemoryConvert, MemoryPowerSupplyRead, MemoryRecall, MemoryScratchpadCopy,
        MemoryScratchpadRead, MemoryScratchpadWrite,
    },
    rom::{AlarmSearch, RomMatch, RomRead, RomSearch, RomSkip},
};

use core::convert::Infallible;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};

/// Alias for `InputPin` + `OutputPin` + `ErrorType`.
pub trait Pin: InputPin + OutputPin + ErrorType<Error = Infallible> {}

impl<T> Pin for T where T: InputPin + OutputPin + ErrorType<Error = Infallible> {}

mod memory;
mod rom;
