//! Blinks an LED
//!
//! This assumes that an LED is connected to pb0

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_halt as _;

use nb::block;

use cortex_m_rt::entry;
use gd32e103_hal::{pac, prelude::*, timer::Timer};

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw rcu and flash devices and convert them into the corresponding HAL
    // structs.
    let mut flash = dp.FMC.constrain();
    let mut rcu = dp.RCU.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcu.cfgr.freeze(&mut flash.ws);

    // Acquire the GPIOB peripheral
    let mut gpiob = dp.GPIOB.split(&mut rcu.apb2);

    // Configure gpio B pin 0 as a push-pull output. The `crl` register is passed to the function
    // in order to configure the port. For pins 8+, crh should be passed instead.
    let mut led = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).start_count_down(1.hz());

    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        block!(timer.wait()).unwrap();
        led.set_high();
        block!(timer.wait()).unwrap();
        led.set_low();
    }
}
