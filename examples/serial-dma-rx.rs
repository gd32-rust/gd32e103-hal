//! Serial interface DMA RX transfer test

#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_semihosting as _;

use cortex_m::{asm, singleton};

use cortex_m_rt::entry;
use gd32e103_hal::{
    pac,
    prelude::*,
    serial::{Config, Serial},
};

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let mut flash = p.FMC.constrain();
    let mut rcu = p.RCU.constrain();

    let clocks = rcu.cfgr.freeze(&mut flash.ws);

    let channels = p.DMA0.split(&mut rcu.ahb);

    let mut gpioa = p.GPIOA.split(&mut rcu.apb2);

    // USART0
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10.into_alternate_push_pull(&mut gpioa.crh);

    // USART1
    // let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    // let rx = gpioa.pa3.into_alternate_push_pull(&mut gpioa.crl);

    let serial = Serial::usart(
        p.USART0,
        (tx, rx),
        Config::default().baudrate(9_600.bps()),
        clocks,
        &mut rcu.apb2,
    );

    let rx = serial.split().1.with_dma(channels.2);
    let buf = singleton!(: [u8; 8] = [0; 8]).unwrap();

    let (_buf, _rx) = rx.read(buf).wait();

    asm::bkpt();

    #[allow(clippy::empty_loop)]
    loop {}
}
