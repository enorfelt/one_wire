# One Wire

A Rust implementation of the 1-Wire master communication protocol. The basic
operations of a 1-Wire bus are: `Reset`, `Write` bit (`1` or `0`) and `Read`
bit. It also implements functions derived from multiple calls to the bit
operations. The time values provided produce the most robust 1-Wire master for
communication with all 1-Wire devices over various line conditions.

## See also

- [1-Wire](https://www.analog.com/en/technical-articles/1wire-communication-through-software.html)
- [embedded-hal](https://github.com/rust-embedded/embedded-hal)

<!-- ## Quick Start

These examples omit error handling to keep them short. You should check all
results and handle them appropriately.

The 1-wire bus requires a single digital pin that is configured as an
open-drain output (it's either open, or connected to ground), and the bus
should have a ~5K Ohm pull-up resistor connected. How you obtain this pin from your
specific device is up the the embedded-hal implementation for that device, but it must
implement both `InputPin` and `OutputPin` 

```rust
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use core::fmt::{Debug, Write};
use one_wire_bus::OneWire;

fn find_devices<P, E>(
    delay: &mut impl DelayUs<u16>,
    tx: &mut impl Write,
    one_wire_pin: P,
)
    where
        P: OutputPin<Error=E> + InputPin<Error=E>,
        E: Debug
{
    let mut one_wire_bus = OneWire::new(one_wire_pin).unwrap();
    for device_address in one_wire_bus.devices(false, delay) {
        // The search could fail at any time, so check each result. The iterator automatically
        // ends after an error.
        let device_address = device_address.unwrap();

        // The family code can be used to identify the type of device
        // If supported, another crate can be used to interact with that device at the given address
        writeln!(tx, "Found device at address {:?} with family code: {:#x?}",
                 device_address, device_address.family_code()).unwrap();
    }
}
```

Example Output
```
Found device at address E800000B1FCD1028 with family code: 0x28
Found device at address 70000008AC851628 with family code: 0x28
Found device at address 0B00000B20687E28 with family code: 0x28
Found device at address 5700000B2015FF28 with family code: 0x28
``` -->
