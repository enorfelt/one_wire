use super::Pin;
use crate::{crc8::check, Command, Error, OneWireDriver, Result, Rom};
use core::convert::Infallible;
use embedded_hal::delay::DelayNs;

pub const COMMAND_ALARM_SEARCH: u8 = 0xEC;
pub const COMMAND_ROM_READ: u8 = 0x33;
pub const COMMAND_ROM_MATCH: u8 = 0x55;
pub const COMMAND_ROM_SKIP: u8 = 0xCC;
pub const COMMAND_ROM_SEARCH: u8 = 0xF0;

const CONFLICT: (bool, bool) = (false, false);
const ZERO: (bool, bool) = (false, true);
const ONE: (bool, bool) = (true, false);
const NONE: (bool, bool) = (true, true);

/// Alarm search command
///
/// When a system is initially brought up, the bus master might not know the
/// number of devices on the 1-Wire bus or their 64-bit ROM codes. The search
/// ROM command allows the bus master to use a process of elimination to
/// identify the 64-bit ROM codes of all slave devices on the bus.
#[derive(Clone, Copy, Debug)]
pub struct AlarmSearch;

/// Read ROM command
///
/// This command allows the bus master to read the DS18B20's 8-bit family code,
/// unique 48-bit serial number, and 8-bit CRC. This command can only be used if
/// there is a single DS18B20 on the bus. If more than one slave is present on
/// the bus, a data collision will occur when all slaves try to transmit at the
/// same time (open drain will produce a wired AND result).
#[derive(Clone, Copy, Debug)]
pub struct RomRead;

impl Command for RomRead {
    type Output = Result<Rom>;

    fn execute(&self, driver: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Self::Output {
        if !driver.reset()? {
            return Err(Error::NoAttachedDevices);
        }
        driver.write_byte(COMMAND_ROM_READ)?;
        let mut rom_bytes = [0u8; 8];
        driver.read_bytes(&mut rom_bytes)?;
        rom_bytes.try_into()
    }
}

/// Match ROM command
///
/// This command allows the bus master to read the DS18B20’s 8-bit family code,
/// unique 48-bit serial number, and 8-bit CRC. This command can only be used if
/// there is a single DS18B20 on the bus. If more than one slave is present on
/// the bus, a data collision will occur when all slaves try to transmit at the
/// same time (open drain will produce a wired AND result).
#[derive(Clone, Copy, Debug)]
pub struct RomMatch {
    pub rom: Rom,
}

impl Command for RomMatch {
    type Output = Result<(), Infallible>;

    fn execute(&self, driver: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Self::Output {
        driver.write_byte(COMMAND_ROM_MATCH)?;
        driver.write_bytes(&Into::<[u8; 8]>::into(self.rom))?;
        Ok(())
    }
}

/// Skip ROM command
///
/// The match ROM command, followed by a 64-bit ROM sequence, allows the bus
/// master to address a specific DS18B20 on a multidrop bus. Only the DS18B20
/// that exactly matches the 64-bit ROM sequence will respond to the following
/// memory function command. All slaves that do not match the 64-bit ROM
/// sequence will wait for a reset pulse. This command can be used with a single
/// or multiple devices on the bus.
#[derive(Clone, Copy, Debug, Default)]
pub struct RomSkip;

impl Command for RomSkip {
    type Output = Result<(), Infallible>;

    fn execute(&self, driver: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Self::Output {
        driver.write_byte(COMMAND_ROM_SKIP)?;
        Ok(())
    }
}

/// Search ROM command
///
/// This command can save time in a single drop bus system by allowing the bus
/// master to access the memory functions without providing the 64-bit ROM code.
/// If more than one slave is present on the bus and a Read command is issued
/// following the Skip ROM command, data collision will occur on the bus as
/// multiple slaves transmit simultaneously (open drain pulldowns will produce a
/// wired AND result).
#[derive(Clone, Copy, Debug, Default)]
pub struct RomSearch {
    conflicts: u64,
}

impl Command for RomSearch {
    type Output = Result<Rom>;

    fn execute(&self, driver: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Self::Output {
        if !driver.reset()? {
            return Err(Error::NoAttachedDevices);
        }
        driver.write_byte(COMMAND_ROM_SEARCH)?;
        let mut rom = 0;
        
        for index in 0..u64::BITS {
            let mask = 1u64 << index;
            let bit1 = driver.read_bit()?;
            let bit2 = driver.read_bit()?;
            
            match (bit1, bit2) {
                // `00`: There are devices attached which have conflicting bits
                CONFLICT => {
                    // For simplicity in a basic search, choose 0 for conflicts
                    // A full search would track discrepancies for multiple devices
                    rom &= !mask;
                    driver.write_bit(false)?;
                }
                // `01`: All devices have a 0-bit in this position
                ZERO => {
                    rom &= !mask;
                    driver.write_bit(false)?;
                }
                // `10`: All devices have a 1-bit in this position
                ONE => {
                    rom |= mask;
                    driver.write_bit(true)?;
                }
                // `11`: No devices are responding
                NONE => return Err(Error::NoAttachedDevices),
            }
        }
        check(&rom.to_le_bytes())?;
        rom.try_into()
    }
}

impl RomSearch {
    fn search(&mut self, one_wire: &mut OneWireDriver<impl Pin, impl DelayNs>) -> Result<Rom> {
        if !one_wire.reset()? {
            return Err(Error::NoAttachedDevices);
        }
        one_wire.write_byte(COMMAND_ROM_SEARCH)?;
        let mut code = 0;
        for index in 0..u64::BITS {
            let mask = 1u64 << index;
            match (one_wire.read_bit()?, one_wire.read_bit()?) {
                // `0b00`: There are still devices attached which have
                // conflicting bits in this position.
                CONFLICT => {
                    // TODO:
                    // discrepancies |= mask;
                    // state.index = index;
                    // self.conflicts ^= mask;
                    self.conflicts ^= mask;
                    if self.conflicts ^ mask == 0 {
                        self.conflicts |= mask;
                        code &= !mask;
                        one_wire.write_bit(false)?;
                    } else {
                        self.conflicts &= !mask;
                        code |= mask;
                        one_wire.write_bit(true)?
                    }
                }
                // `0b01`: All devices still coupled have a 0-bit in this bit
                // position.
                ZERO => {
                    code |= mask;
                    one_wire.write_bit(false)?;
                }
                // `0b10`: All devices still coupled have a 1-bit in this bit
                // position.
                ONE => {
                    code &= !mask;
                    one_wire.write_bit(true)?;
                }
                // `0b11`: There are no devices attached to the 1-Wire bus.
                NONE => return Err(Error::NoAttachedDevices),
            }
        }
        code.try_into()
    }
}

pub struct Iter<'a, T, U> {
    driver: &'a mut OneWireDriver<T, U>,
    discrepancies: u64,
    index: u8,
}

impl<T, U> Iterator for Iter<'_, T, U> {
    type Item = Result<Rom>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

// /// Search for device addresses on the bus
// ///
// /// They can be filtered to only alarming devices if needed Start the first
// /// search with a search_state of `None`, then use the returned state for
// /// subsequent searches There is no time limit for continuing a search, but
// /// if devices are added / removed / change alarm state, the search may
// /// return an error or fail to find a device Device addresses will always be
// /// returned in the same order (lowest to highest, Little Endian)
// pub fn device_search(
//     &mut self,
//     search_state: Option<&SearchState>,
//     only_alarming: bool,
// ) -> Result<Option<(Address, SearchState)>> {
//     if let Some(search_state) = search_state {
//         if search_state.discrepancies == 0 {
//             return Ok(None);
//         }
//     }

//     if !self.reset()? {
//         return Ok(None);
//     }
//     if only_alarming {
//         self.write_byte(COMMAND_ALARM_SEARCH)?;
//     } else {
//         self.write_byte(COMMAND_ROM_SEARCH)?;
//     }

//     let mut last_discrepancy_index: u8 = 0;
//     let mut address;
//     let mut discrepancies;
//     let continue_start_bit;

//     if let Some(search_state) = search_state {
//         // follow up to the last discrepancy
//         for bit_index in 0..search_state.last_discrepancy_index {
//             let _false_bit = !self.read_bit()?;
//             let _true_bit = !self.read_bit()?;
//             let was_discrepancy_bit =
//                 (search_state.discrepancies & (1_u64 << (bit_index as u64))) != 0;
//             if was_discrepancy_bit {
//                 last_discrepancy_index = bit_index;
//             }
//             let previous_chosen_bit = (search_state.address & (1_u64 << (bit_index as u64))) != 0;

//             // choose the same as last time
//             self.write_bit(previous_chosen_bit)?;
//         }
//         address = search_state.address;
//         // This is the discrepancy bit. False is always chosen to start, so choose true this time
//         {
//             let false_bit = !self.read_bit()?;
//             let true_bit = !self.read_bit()?;
//             if !(false_bit && true_bit) {
//                 // A different response was received than last search
//                 return Err(Error::UnexpectedResponse);
//             }
//             let address_mask = 1_u64 << (search_state.last_discrepancy_index as u64);
//             address |= address_mask;
//             self.write_bit(true)?;
//         }

//         //keep all discrepancies except the last one
//         discrepancies =
//             search_state.discrepancies & !(1_u64 << (search_state.last_discrepancy_index as u64));
//         continue_start_bit = search_state.last_discrepancy_index + 1;
//     } else {
//         address = 0;
//         discrepancies = 0;
//         continue_start_bit = 0;
//     }
//     for bit_index in continue_start_bit..64 {
//         let false_bit = !self.read_bit()?;
//         let true_bit = !self.read_bit()?;
//         let chosen_bit = match (false_bit, true_bit) {
//             (false, false) => {
//                 // No devices responded to the search request
//                 return Err(Error::UnexpectedResponse);
//             }
//             (false, true) => {
//                 // All remaining devices have the true bit set
//                 true
//             }
//             (true, false) => {
//                 // All remaining devices have the false bit set
//                 false
//             }
//             (true, true) => {
//                 // Discrepancy, multiple values reported
//                 // choosing the lower value here
//                 discrepancies |= 1_u64 << (bit_index as u64);
//                 last_discrepancy_index = bit_index;
//                 false
//             }
//         };
//         let address_mask = 1_u64 << (bit_index as u64);
//         if chosen_bit {
//             address |= address_mask;
//         } else {
//             address &= !address_mask;
//         }
//         self.write_bit(chosen_bit)?;
//     }
//     check(&address.to_le_bytes())?;
//     Ok(Some((
//         Address(address),
//         SearchState {
//             address,
//             discrepancies,
//             last_discrepancy_index,
//         },
//     )))
// }

// /// Devices
// pub struct Devices<'a, T, U> {
//     one_wire: &'a mut OneWire<T, U>,
//     state: Option<SearchState>,
//     finished: bool,
//     only_alarming: bool,
// }

// impl<'a, T: Pin, D: DelayUs> Iterator for Devices<'a, T, D> {
//     type Item = Result<Address>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.finished {
//             return None;
//         }
//         let result = self
//             .one_wire
//             .device_search(self.state.as_ref(), self.only_alarming);
//         match result {
//             Ok(Some((address, search_state))) => {
//                 self.state = Some(search_state);
//                 Some(Ok(address))
//             }
//             Ok(None) => {
//                 self.state = None;
//                 self.finished = true;
//                 None
//             }
//             Err(error) => {
//                 self.state = None;
//                 self.finished = true;
//                 Some(Err(error))
//             }
//         }
//     }
// }
