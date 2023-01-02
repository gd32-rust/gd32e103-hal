//! Blinks several LEDs stored in an array

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_semihosting as _;

use nb::block;

use cortex_m_rt::entry;
use gd32e103_hal::{pac, prelude::*, timer::Timer};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut rcu = dp.RCU.constrain();
    let mut flash = dp.FMC.constrain();

    let clocks = rcu.cfgr.freeze(&mut flash.ws);

    // Acquire the GPIO peripherals
    let mut gpioa = dp.GPIOA.split(&mut rcu.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcu.apb2);

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).start_count_down(1.hz());

    // Create an array of LEDS to blink
    let mut leds = [
        gpioc.pc13.into_push_pull_output(&mut gpioc.crh).erase(),
        gpioa.pa1.into_push_pull_output(&mut gpioa.crl).erase(),
    ];

    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        block!(timer.wait()).unwrap();
        for led in leds.iter_mut() {
            led.set_high();
        }
        block!(timer.wait()).unwrap();
        for led in leds.iter_mut() {
            led.set_low();
        }
    }
}
