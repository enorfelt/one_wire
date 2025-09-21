pub use self::{
    memory::{
        MemoryConvert, MemoryPowerSupplyRead, MemoryRecall, MemoryScratchpadCopy,
        MemoryScratchpadRead, MemoryScratchpadWrite,
    },
    rom::{AlarmSearch, RomMatch, RomRead, RomSearch, RomSkip, COMMAND_ALARM_SEARCH, COMMAND_ROM_SEARCH},
};

use core::convert::Infallible;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};

/// Alias for `InputPin` + `OutputPin` + `ErrorType`.
pub trait Pin: InputPin + OutputPin + ErrorType<Error = Infallible> {}

impl<T> Pin for T where T: InputPin + OutputPin + ErrorType<Error = Infallible> {}

mod memory;
mod rom;
