//! # Controller Area Network (CAN) Interface
//!
//! ## Alternate function remapping
//!
//! TX: Alternate Push-Pull Output
//! RX: Input Floating Input
//!
//! ### CAN0
//!
//! | Function | NoRemap |     Remap     |
//! |----------|---------|---------------|
//! | TX       | PA12    | PB9   | PD1   |
//! | RX       | PA11    | PB8   | PD0   |
//!
//! ### CAN1
//!
//! | Function | NoRemap | Remap |
//! |----------|---------|-------|
//! | TX       | PB13    | PB6   |
//! | RX       | PB12    | PB5   |

use crate::gpio::{
    gpioa::{PA11, PA12},
    gpiob::{PB12, PB13, PB5, PB6, PB8, PB9},
    gpiod::{PD0, PD1},
    Alternate, Floating, Input, PushPull,
};
use crate::pac::{AFIO, CAN0, CAN1};
use crate::rcu::APB1;

mod sealed {
    pub trait Sealed {}
}

pub trait Pins: sealed::Sealed {
    type Instance;
    fn remap(afio: &mut AFIO);
}

impl sealed::Sealed for (PA12<Alternate<PushPull>>, PA11<Input<Floating>>) {}
impl Pins for (PA12<Alternate<PushPull>>, PA11<Input<Floating>>) {
    type Instance = CAN0;

    fn remap(afio: &mut AFIO) {
        afio.pcf0.modify(|_, w| unsafe { w.can0_remap().bits(0) });
    }
}

impl sealed::Sealed for (PB9<Alternate<PushPull>>, PB8<Input<Floating>>) {}
impl Pins for (PB9<Alternate<PushPull>>, PB8<Input<Floating>>) {
    type Instance = CAN0;

    fn remap(afio: &mut AFIO) {
        afio.pcf0
            .modify(|_, w| unsafe { w.can0_remap().bits(0b10) });
    }
}

impl sealed::Sealed for (PD1<Alternate<PushPull>>, PD0<Input<Floating>>) {}
impl Pins for (PD1<Alternate<PushPull>>, PD0<Input<Floating>>) {
    type Instance = CAN0;

    fn remap(afio: &mut AFIO) {
        afio.pcf0
            .modify(|_, w| unsafe { w.can0_remap().bits(0b11) });
    }
}

impl sealed::Sealed for (PB13<Alternate<PushPull>>, PB12<Input<Floating>>) {}
impl Pins for (PB13<Alternate<PushPull>>, PB12<Input<Floating>>) {
    type Instance = CAN1;

    fn remap(afio: &mut AFIO) {
        afio.pcf0.modify(|_, w| w.can1_remap().clear_bit());
    }
}

impl sealed::Sealed for (PB6<Alternate<PushPull>>, PB5<Input<Floating>>) {}
impl Pins for (PB6<Alternate<PushPull>>, PB5<Input<Floating>>) {
    type Instance = CAN1;

    fn remap(afio: &mut AFIO) {
        afio.pcf0.modify(|_, w| w.can1_remap().set_bit());
    }
}

/// Interface to the CAN peripheral.
pub struct Can<Instance> {
    _peripheral: Instance,
}

impl<Instance> Can<Instance>
where
    Instance: crate::rcu::Enable<Bus = APB1>,
{
    /// Creates a CAN interaface.
    pub fn new(can: Instance, apb: &mut APB1) -> Can<Instance> {
        Instance::enable(apb);
        Can { _peripheral: can }
    }

    /// Routes CAN TX signals and RX signals to pins.
    pub fn assign_pins<P>(&self, _pins: P, afio: &mut AFIO)
    where
        P: Pins<Instance = Instance>,
    {
        P::remap(afio);
    }
}

unsafe impl bxcan::Instance for Can<CAN0> {
    const REGISTERS: *mut bxcan::RegisterBlock = CAN0::ptr() as *mut _;
}

unsafe impl bxcan::Instance for Can<CAN1> {
    const REGISTERS: *mut bxcan::RegisterBlock = CAN1::ptr() as *mut _;
}

unsafe impl bxcan::FilterOwner for Can<CAN0> {
    const NUM_FILTER_BANKS: u8 = 28;
}

unsafe impl bxcan::MasterInstance for Can<CAN0> {}
