impl<T: Pin, U: DelayUs> OneWire<T, U> {
    pub fn into_inner(self) -> T {
        self.pin
    }

    fn wait_for_high(&mut self) -> Result<()> {
        // wait up to 250 µs for the bus to become high (from the pull-up resistor)
        for _ in 0..125 {
            if self.is_high()? {
                return Ok(());
            }
            self.wait(2);
        }
        Err(Error::NotHigh)
    }

    /// Returns an iterator that iterates over all device addresses on the bus.
    ///
    /// They can be filtered to only alarming devices if needed There is no
    /// requirement to immediately finish iterating all devices, but if devices
    /// are added / removed / change alarm state, the search may return an error
    /// or fail to find a device Device addresses will always be returned in the
    /// same order (lowest to highest, Little Endian)
    pub fn devices(&mut self, only_alarming: bool) -> Devices<T, U> {
        Devices {
            one_wire: self,
            state: None,
            finished: false,
            only_alarming,
        }
    }

    /// Search for device addresses on the bus
    ///
    /// They can be filtered to only alarming devices if needed Start the first
    /// search with a search_state of `None`, then use the returned state for
    /// subsequent searches There is no time limit for continuing a search, but
    /// if devices are added / removed / change alarm state, the search may
    /// return an error or fail to find a device Device addresses will always be
    /// returned in the same order (lowest to highest, Little Endian)
    pub fn device_search(
        &mut self,
        search_state: Option<&SearchState>,
        only_alarming: bool,
    ) -> Result<Option<(Address, SearchState)>> {
        if let Some(search_state) = search_state {
            if search_state.discrepancies == 0 {
                return Ok(None);
            }
        }

        if !self.reset()? {
            return Ok(None);
        }
        if only_alarming {
            self.write_byte(COMMAND_ALARM_SEARCH)?;
        } else {
            self.write_byte(COMMAND_ROM_SEARCH)?;
        }

        let mut last_discrepancy_index: u8 = 0;
        let mut address;
        let mut discrepancies;
        let continue_start_bit;

        if let Some(search_state) = search_state {
            // follow up to the last discrepancy
            for bit_index in 0..search_state.last_discrepancy_index {
                let _false_bit = !self.read_bit()?;
                let _true_bit = !self.read_bit()?;
                let was_discrepancy_bit =
                    (search_state.discrepancies & (1_u64 << (bit_index as u64))) != 0;
                if was_discrepancy_bit {
                    last_discrepancy_index = bit_index;
                }
                let previous_chosen_bit =
                    (search_state.address & (1_u64 << (bit_index as u64))) != 0;

                // choose the same as last time
                self.write_bit(previous_chosen_bit)?;
            }
            address = search_state.address;
            // This is the discrepancy bit. False is always chosen to start, so choose true this time
            {
                let false_bit = !self.read_bit()?;
                let true_bit = !self.read_bit()?;
                if !(false_bit && true_bit) {
                    // A different response was received than last search
                    return Err(Error::UnexpectedResponse);
                }
                let address_mask = 1_u64 << (search_state.last_discrepancy_index as u64);
                address |= address_mask;
                self.write_bit(true)?;
            }

            //keep all discrepancies except the last one
            discrepancies = search_state.discrepancies
                & !(1_u64 << (search_state.last_discrepancy_index as u64));
            continue_start_bit = search_state.last_discrepancy_index + 1;
        } else {
            address = 0;
            discrepancies = 0;
            continue_start_bit = 0;
        }
        for bit_index in continue_start_bit..64 {
            let false_bit = !self.read_bit()?;
            let true_bit = !self.read_bit()?;
            let chosen_bit = match (false_bit, true_bit) {
                (false, false) => {
                    // No devices responded to the search request
                    return Err(Error::UnexpectedResponse);
                }
                (false, true) => {
                    // All remaining devices have the true bit set
                    true
                }
                (true, false) => {
                    // All remaining devices have the false bit set
                    false
                }
                (true, true) => {
                    // Discrepancy, multiple values reported
                    // choosing the lower value here
                    discrepancies |= 1_u64 << (bit_index as u64);
                    last_discrepancy_index = bit_index;
                    false
                }
            };
            let address_mask = 1_u64 << (bit_index as u64);
            if chosen_bit {
                address |= address_mask;
            } else {
                address &= !address_mask;
            }
            self.write_bit(chosen_bit)?;
        }
        crc::check_crc8(&address.to_le_bytes())?;
        Ok(Some((
            Address(address),
            SearchState {
                address,
                discrepancies,
                last_discrepancy_index,
            },
        )))
    }
}

/// Devices
pub struct Devices<'a, T, D> {
    one_wire: &'a mut OneWire<T, D>,
    state: Option<SearchState>,
    finished: bool,
    only_alarming: bool,
}

impl<'a, T, D> Iterator for Devices<'a, T, D>
where
    T: Pin,
    D: DelayUs,
{
    type Item = Result<Address>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let result = self
            .one_wire
            .device_search(self.state.as_ref(), self.only_alarming);
        match result {
            Ok(Some((address, search_state))) => {
                self.state = Some(search_state);
                Some(Ok(address))
            }
            Ok(None) => {
                self.state = None;
                self.finished = true;
                None
            }
            Err(err) => {
                self.state = None;
                self.finished = true;
                Some(Err(err))
            }
        }
    }
}

#[derive(Debug)]
pub struct SearchState {
    // The address of the last found device
    address: u64,

    // bitflags of discrepancies found
    discrepancies: u64,

    // index of the last (leftmost / closest to MSB) discrepancy bit. This can
    // be calculated from the discrepancy bitflags, but it's cheaper to just
    // save it. Index is an offset from the LSB
    last_discrepancy_index: u8,
}
