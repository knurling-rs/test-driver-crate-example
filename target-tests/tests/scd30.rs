#![no_std]
#![no_main]

use defmt_rtt as _; // defmt transport
use nrf52840_hal as _; // memory layout
use panic_probe as _; // panic handler

use nrf52840_hal::{pac::TWIM0, twim::Twim};
use scd30::Scd30;

struct State {
    scd30: Scd30<Twim<TWIM0>>,
}

#[defmt_test::tests]
mod tests {
    use defmt::{assert_eq, unwrap};
    use nrf52840_hal::{
        gpio::p0,
        twim::{self, Twim},
    };

    use super::State;

    #[init]
    fn setup() -> State {
        // enable and reset the cycle counter
        let mut core = unwrap!(cortex_m::Peripherals::take());
        core.DCB.enable_trace();
        unsafe { core.DWT.cyccnt.write(0) }
        core.DWT.enable_cycle_counter();
        defmt::timestamp!("{=u32:µs}", cortex_m::peripheral::DWT::get_cycle_count());

        // initialize I2C
        let peripherals = unwrap!(nrf52840_hal::pac::Peripherals::take());
        let pins = p0::Parts::new(peripherals.P0);
        let scl = pins.p0_30.into_floating_input().degrade();
        let sda = pins.p0_31.into_floating_input().degrade();
        let pins = twim::Pins { scl, sda };
        let i2c = Twim::new(peripherals.TWIM0, pins, twim::Frequency::K100);

        let scd30 = scd30::Scd30::init(i2c);
        State { scd30 }
    }

    #[test]
    fn confirm_firmware_id(state: &mut State) {
        const EXPECTED: [u8; 2] = [3, 66];
        let firmware_id = state.scd30.get_firmware_version().unwrap();
        assert_eq!(EXPECTED, firmware_id);
    }
}
