//! Turns an LED on
//!
//! This assumes that an active high LED is connected to pc9.

#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_semihosting as _;

use cortex_m_rt::entry;
use gd32e103_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let mut rcu = p.RCU.constrain();
    let mut gpioc = p.GPIOC.split(&mut rcu.apb2);

    gpioc.pc9.into_push_pull_output(&mut gpioc.crh).set_high();

    #[allow(clippy::empty_loop)]
    loop {}
}
