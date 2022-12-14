//! Prints "Hello, world" on the OpenOCD console

#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_semihosting as _;

use cortex_m_semihosting::hprintln;
use gd32e103_hal as _;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    hprintln!("Hello, world!");

    #[allow(clippy::empty_loop)]
    loop {}
}
