//! CRC calculation

#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_semihosting as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use gd32e103_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let mut rcu = p.RCU.constrain();
    let mut crc = p.CRC.constrain(&mut rcu.ahb);

    crc.reset();
    crc.write(0x12345678);

    let val = crc.read();
    hprintln!("found={:08x}, expected={:08x}", val, 0xdf8a8a2bu32);

    #[allow(clippy::empty_loop)]
    loop {}
}
