use super::{Pin, RomMatch, RomSkip};
use crate::{
    command::Commander,
    error::{Error, Result},
    scratchpad::Scratchpad,
    Command, OneWireDriver, Rom,
};
use embedded_hal::delay::DelayNs;

pub const COMMAND_MEMORY_CONVERT: u8 = 0x44;
pub const COMMAND_MEMORY_RECALL: u8 = 0xB8;
pub const COMMAND_MEMORY_POWER_SUPPLY_READ: u8 = 0xB4;
pub const COMMAND_MEMORY_SCRATCHPAD_COPY: u8 = 0x48;
pub const COMMAND_MEMORY_SCRATCHPAD_READ: u8 = 0xBE;
pub const COMMAND_MEMORY_SCRATCHPAD_WRITE: u8 = 0x4E;

const READ_SLOT_DURATION_MICROS: u16 = 70;

/// Initiates temperature conversion.
///
/// You should wait for the measurement to finish before reading the
/// measurement. The amount of time you need to wait depends on the current
/// resolution configuration
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryConvert {
    pub rom: Option<Rom>,
}

impl Command for MemoryConvert {
    type Output = Result<()>;

    fn execute(&self, driver: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Self::Output {
        driver.reset()?;
        match self.rom {
            Some(rom) => driver.run(RomMatch { rom })?,
            None => driver.run(RomSkip)?,
        }
        driver.write_byte(COMMAND_MEMORY_CONVERT)?;
        Ok(())
    }
}

/// Signals the mode of DS18B20 power supply to the master.
#[derive(Clone, Copy, Debug)]
pub enum MemoryPowerSupplyRead {
    /// Signals the mode of DS18B20 power supply to the master.
    Read,
}

/// Recalls values stored in nonvolatile memory (EEPROM) into scratchpad
/// (temperature triggers). Load config from EEPROM to scratchpad.
///
/// If `rom` is `None` - for all devices simultaneously.
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryRecall {
    pub rom: Option<Rom>,
}

impl Command for MemoryRecall {
    type Output = Result<()>;

    fn execute(&self, driver: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Self::Output {
        driver.reset()?;
        match self.rom {
            Some(rom) => driver.run(RomMatch { rom })?,
            None => driver.run(RomSkip)?,
        }
        driver.write_byte(COMMAND_MEMORY_RECALL)?;
        // wait for the recall to finish (up to 10ms)
        let max_retries = (10000 / READ_SLOT_DURATION_MICROS) + 1;
        for _ in 0..max_retries {
            if driver.read_bit()? == true {
                return Ok(());
            }
        }
        Err(Error::Timeout)
    }
}

/// Copies scratchpad into nonvolatile memory (EEPROM) (addresses 2 through 4
/// only). Save config from scratchpad to EEPROM.
///
/// If `rom` is `None` - for all devices simultaneously.
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryScratchpadCopy {
    pub rom: Option<Rom>,
}

impl Command for MemoryScratchpadCopy {
    type Output = Result<()>;

    fn execute(&self, driver: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Self::Output {
        driver.reset()?;
        match self.rom {
            Some(rom) => driver.run(RomMatch { rom })?,
            None => driver.run(RomSkip)?,
        }
        driver.write_byte(COMMAND_MEMORY_SCRATCHPAD_COPY)?;
        driver.wait(10000); // delay 10ms for the write to complete
        Ok(())
    }
}

/// Reads bytes from scratchpad and reads CRC byte.
#[derive(Clone, Copy, Debug)]
pub struct MemoryScratchpadRead {
    pub rom: Rom,
}

impl Command for MemoryScratchpadRead {
    type Output = Result<Scratchpad>;

    fn execute(&self, driver: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Self::Output {
        driver.reset()?;
        driver.run(RomMatch { rom: self.rom })?;
        driver.write_byte(COMMAND_MEMORY_SCRATCHPAD_READ)?;
        let mut bytes = [0; 9];
        driver.read_bytes(&mut bytes)?;
        bytes.try_into()
    }
}

/// Writes bytes into scratchpad at addresses 2 through 4 (TH and TL
/// temperature triggers and config).
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryScratchpadWrite {
    pub rom: Option<Rom>,
    pub scratchpad: Scratchpad,
}

impl Command for MemoryScratchpadWrite {
    type Output = Result<()>;

    fn execute(&self, driver: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Self::Output {
        driver.reset()?;
        match self.rom {
            Some(rom) => driver.run(RomMatch { rom })?,
            None => driver.run(RomSkip)?,
        }
        driver.write_byte(COMMAND_MEMORY_SCRATCHPAD_WRITE)?;
        driver.write_byte(self.scratchpad.triggers.low as _)?;
        driver.write_byte(self.scratchpad.triggers.high as _)?;
        driver.write_byte(self.scratchpad.configuration.resolution as _)?;
        Ok(())
    }
}

/// And command
#[derive(Clone, Copy, Debug, Default)]
pub struct And<T, U>(pub T, pub U);

// impl<T: Command<Output = V>, U: Command<Output = V>, V> Command for And<T, U> {
//     type Output = Result<()>;

//     fn execute(&self, one_wire: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Self::Output {
//         one_wire.reset()?;
//         one_wire.run(self.0)?;
//         one_wire.run(self.1)?;
//         Ok(())
//     }
// }

// /// Sends a reset, followed with either a SKIP_ROM or MATCH_ROM (with an
// /// address), and then the supplied command This should be followed by any
// /// reading/writing, if needed by the command used.
// #[derive(Clone, Copy, Debug)]
// pub enum MatchOrSkip {
//     Match { address: Address },
//     Skip,
// }
// impl Command for MatchOrSkip {
//     type Output = Result<()>;
//     fn execute(&self, one_wire: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Self::Output {
//         one_wire.reset()?;
//         match *self {
//             Self::Match { address } => {
//                 one_wire.run(Match { address })?;
//             }
//             Self::Skip => {
//                 one_wire.run(Skip)?;
//             }
//         }
//         Ok(())
//     }
// }
