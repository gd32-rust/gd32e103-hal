//! "Blinky" using delays instead of a timer

#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_semihosting as _;

use cortex_m_rt::entry;
use gd32e103_hal::{delay::Delay, pac, prelude::*};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcu = dp.RCU.constrain();
    let mut flash = dp.FMC.constrain();

    let clocks = rcu.cfgr.freeze(&mut flash.ws);

    let mut gpioc = dp.GPIOC.split(&mut rcu.apb2);

    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    let mut delay = Delay::new(cp.SYST, clocks);

    loop {
        led.set_high();
        delay.delay_ms(1_000_u16);
        led.set_low();
        delay.delay_ms(1_000_u16);
    }
}
