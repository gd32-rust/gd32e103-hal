//! Disables the JTAG ports to give access to pb3, pb4 and PA15

#![no_main]
#![no_std]

use panic_semihosting as _;

use cortex_m_rt::entry;
use gd32e103_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let mut rcu = p.RCU.constrain();
    let mut gpioa = p.GPIOA.split(&mut rcu.apb2);

    // If you really want to use a JTAG pin for something else, you must first
    // Disable JTAG in AFIO
    let (mut pa13, mut pa14) = unsafe {
        p.AFIO.pcf0.write(|w| w.swj_cfg().bits(0b100));
        (
            gpioa.pa13.activate().into_push_pull_output(&mut gpioa.crh),
            gpioa.pa14.activate().into_push_pull_output(&mut gpioa.crh),
        )
    };

    loop {
        pa13.toggle();
        pa14.toggle();
        cortex_m::asm::delay(8_000_000);
    }
}
