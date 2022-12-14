//! Serial interface circular DMA RX transfer test

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_semihosting as _;

use cortex_m::{asm, singleton};

use cortex_m_rt::entry;
use gd32e103_hal::{
    dma::Half,
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
    let buf = singleton!(: [[u8; 8]; 2] = [[0; 8]; 2]).unwrap();

    let mut circ_buffer = rx.circ_read(buf);

    while circ_buffer.readable_half().unwrap() != Half::First {}

    let _first_half = circ_buffer.peek(|half, _| *half).unwrap();

    while circ_buffer.readable_half().unwrap() != Half::Second {}

    let _second_half = circ_buffer.peek(|half, _| *half).unwrap();

    asm::bkpt();

    #[allow(clippy::empty_loop)]
    loop {}
}
