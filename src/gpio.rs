// Copyright 2022 The gd32f1x0-hal authors.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::convert::Infallible;
use core::marker::PhantomData;

use crate::rcu::APB2;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin};

mod partially_erased;
pub use partially_erased::{PEPin, PartiallyErasedPin};
mod erased;
pub use erased::{EPin, ErasedPin};

/// Slew rates available for Output and relevant AlternateMode Pins
#[derive(Clone, Copy, Debug)]
pub enum Speed {
    Mhz10 = 0b01,
    Mhz2 = 0b10,
    Mhz50 = 0b11,
}

pub trait PinExt {
    type Mode;

    /// Return pin number
    fn pin_id(&self) -> u8;

    /// Return port number
    fn port_id(&self) -> u8;
}

/// Allow setting of the slew rate of an IO pin
///
/// Initially all pins are set to the maximum slew rate
pub trait OutputSpeed: HL {
    fn set_speed(&mut self, ctl: &mut Self::Ctl, speed: Speed);
}

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self, apb2: &mut APB2) -> Self::Parts;
}

/// Marker trait for active states.
pub trait Active {}

/// Input mode (type state)
#[derive(Default)]
pub struct Input<MODE = Floating> {
    _mode: PhantomData<MODE>,
}
impl<MODE> Active for Input<MODE> {}

/// Used by the debugger (type state)
#[derive(Default)]
pub struct Debugger;

/// Floating input (type state)
#[derive(Default)]
pub struct Floating;

/// Pulled down input (type state)
#[derive(Default)]
pub struct PullDown;

/// Pulled up input (type state)
#[derive(Default)]
pub struct PullUp;

/// Output mode (type state)
#[derive(Default)]
pub struct Output<MODE = PushPull> {
    _mode: PhantomData<MODE>,
}
impl<MODE> Active for Output<MODE> {}

/// Push pull output (type state)
#[derive(Default)]
pub struct PushPull;

/// Open drain output (type state)
#[derive(Default)]
pub struct OpenDrain;

/// Analog mode (type state)
#[derive(Default)]
pub struct Analog;
impl Active for Analog {}

/// Alternate function
#[derive(Default)]
pub struct Alternate<MODE = PushPull> {
    _mode: PhantomData<MODE>,
}
impl<MODE> Active for Alternate<MODE> {}

/// Digital output pin state
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PinState {
    High,
    Low,
}

mod sealed {
    pub trait PinMode: Default {
        const CNF: u32;
        const MODE: u32;
        const PULL: Option<bool> = None;
    }
}

use sealed::PinMode;

/// Tracks the current pin state for dynamic pins
pub enum Dynamic {
    InputFloating,
    InputPullUp,
    InputPullDown,
    OutputPushPull,
    OutputOpenDrain,
}

impl Default for Dynamic {
    fn default() -> Self {
        Dynamic::InputFloating
    }
}

impl Active for Dynamic {}

#[derive(Debug, PartialEq, Eq)]
pub enum PinModeError {
    IncorrectMode,
}

impl Dynamic {
    fn is_input(&self) -> bool {
        use Dynamic::*;
        match self {
            InputFloating | InputPullUp | InputPullDown | OutputOpenDrain => true,
            OutputPushPull => false,
        }
    }

    fn is_output(&self) -> bool {
        use Dynamic::*;
        match self {
            InputFloating | InputPullUp | InputPullDown => false,
            OutputPushPull | OutputOpenDrain => true,
        }
    }
}

macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, $PXx:ident, $port_id:expr, [
        $($PXi:ident: ($pxi:ident, $pin_number:expr $(, $MODE:ty)?),)+
    ]) => {
        /// GPIO
        pub mod $gpiox {
            use crate::pac::{$GPIOX};
            use crate::rcu::{APB2, Enable, Reset};
            use super::{Active, Floating, GpioExt, Input, PartiallyErasedPin, ErasedPin, Pin, Ctl};
            #[allow(unused)]
            use super::Debugger;

            /// GPIO parts
            pub struct Parts {
                /// Opaque CRL register
                pub crl: Ctl<$port_id, false>,
                /// Opaque CRH register
                pub crh: Ctl<$port_id, true>,
                $(
                    /// Pin
                    pub $pxi: $PXi $(<$MODE>)?,
                )+
            }

            $(
                pub type $PXi<MODE = Input<Floating>> = Pin<$port_id, $pin_number, MODE>;
            )+

            impl GpioExt for $GPIOX {
                type Parts = Parts;

                fn split(self, apb2: &mut APB2) -> Parts {
                    $GPIOX::enable(apb2);
                    $GPIOX::reset(apb2);

                    Parts {
                        crl: Ctl::<$port_id, false>(()),
                        crh: Ctl::<$port_id, true>(()),
                        $(
                            $pxi: $PXi::new(),
                        )+
                    }
                }
            }

            impl<MODE> PartiallyErasedPin<$port_id, MODE> {
                pub fn erase(self) -> ErasedPin<MODE> {
                    ErasedPin::$PXx(self)
                }
            }

            impl<const N: u8, MODE> Pin<$port_id, N, MODE>
            where
                MODE: Active,
            {
                /// Erases the pin number and port from the type
                ///
                /// This is useful when you want to collect the pins into an array where you
                /// need all the elements to have the same type
                pub fn erase(self) -> ErasedPin<MODE> {
                    self.erase_number().erase()
                }
            }
        }

        pub use $gpiox::{ $($PXi,)+ };
    }
}

/// Generic pin type
///
/// - `P` is port name: `A` for GPIOA, `B` for GPIOB, etc.
/// - `N` is pin number: from `0` to `15`.
/// - `MODE` is one of the pin modes (see [Modes](crate::gpio#modes) section).
pub struct Pin<const P: char, const N: u8, MODE = Input<Floating>> {
    mode: MODE,
}

impl<const P: char, const N: u8, MODE> Pin<P, N, MODE> {
    const OFFSET: u32 = (4 * (N as u32)) % 32;
}

/// Represents high or low configuration register
pub trait HL {
    /// Configuration register associated to pin
    type Ctl;
}

macro_rules! ctl {
    ($ctl_is_h:literal: [$($pin_number:literal),+]) => {
        $(
            impl<const P: char, MODE> HL for Pin<P, $pin_number, MODE> {
                type Ctl = Ctl<P, $ctl_is_h>;
            }
        )+
    }
}

ctl!(false: [0, 1, 2, 3, 4, 5, 6, 7]);
ctl!(true: [8, 9, 10, 11, 12, 13, 14, 15]);

impl<const P: char, const N: u8, MODE: Default> Pin<P, N, MODE> {
    fn new() -> Self {
        Self {
            mode: Default::default(),
        }
    }
}

impl<const P: char, const N: u8, MODE> PinExt for Pin<P, N, MODE> {
    type Mode = MODE;

    #[inline(always)]
    fn pin_id(&self) -> u8 {
        N
    }

    #[inline(always)]
    fn port_id(&self) -> u8 {
        P as u8 - b'A'
    }
}

impl<const P: char, const N: u8> Pin<P, N, Debugger> {
    /// Put the pin in an active state. The caller
    /// must enforce that the pin is really in this
    /// state in the hardware.
    #[allow(dead_code)]
    pub(crate) unsafe fn activate(self) -> Pin<P, N, Input<Floating>> {
        Pin::new()
    }
}

impl<const P: char, const N: u8> OutputPin for Pin<P, N, Dynamic> {
    type Error = PinModeError;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        if self.mode.is_output() {
            self._set_high();
            Ok(())
        } else {
            Err(PinModeError::IncorrectMode)
        }
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        if self.mode.is_output() {
            self._set_low();
            Ok(())
        } else {
            Err(PinModeError::IncorrectMode)
        }
    }
}

impl<const P: char, const N: u8> InputPin for Pin<P, N, Dynamic> {
    type Error = PinModeError;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.is_low().map(|b| !b)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        if self.mode.is_input() {
            Ok(self._is_low())
        } else {
            Err(PinModeError::IncorrectMode)
        }
    }
}

// Internal helper functions

// NOTE: The functions in this impl block are "safe", but they
// are callable when the pin is in modes where they don't make
// sense.
impl<const P: char, const N: u8, MODE> Pin<P, N, MODE> {
    /**
      Set the output of the pin regardless of its mode.
      Primarily used to set the output value of the pin
      before changing its mode to an output to avoid
      a short spike of an incorrect value
    */

    #[inline(always)]
    fn _set_state(&mut self, state: PinState) {
        match state {
            PinState::High => self._set_high(),
            PinState::Low => self._set_low(),
        }
    }

    #[inline(always)]
    fn _set_high(&mut self) {
        // NOTE(unsafe) atomic write to a stateless register
        unsafe { (*Gpio::<P>::ptr()).bop.write(|w| w.bits(1 << N)) }
    }

    #[inline(always)]
    fn _set_low(&mut self) {
        // NOTE(unsafe) atomic write to a stateless register
        unsafe { (*Gpio::<P>::ptr()).bc.write(|w| w.bits(1 << N)) }
    }

    #[inline(always)]
    fn _is_set_low(&self) -> bool {
        // NOTE(unsafe) atomic read with no side effects
        unsafe { (*Gpio::<P>::ptr()).octl.read().bits() & (1 << N) == 0 }
    }

    #[inline(always)]
    fn _is_low(&self) -> bool {
        // NOTE(unsafe) atomic read with no side effects
        unsafe { (*Gpio::<P>::ptr()).istat.read().bits() & (1 << N) == 0 }
    }
}

impl<const P: char, const N: u8, MODE> Pin<P, N, MODE>
where
    MODE: Active,
{
    /// Erases the pin number from the type
    #[inline]
    pub fn erase_number(self) -> PartiallyErasedPin<P, MODE> {
        PartiallyErasedPin::new(N)
    }
}

impl<const P: char, const N: u8, MODE> Pin<P, N, Output<MODE>> {
    #[inline]
    pub fn set_high(&mut self) {
        self._set_high()
    }

    #[inline]
    pub fn set_low(&mut self) {
        self._set_low()
    }

    #[inline(always)]
    pub fn get_state(&self) -> PinState {
        if self._is_set_low() {
            PinState::Low
        } else {
            PinState::High
        }
    }

    #[inline(always)]
    pub fn set_state(&mut self, state: PinState) {
        self._set_state(state)
    }

    #[inline]
    pub fn is_set_high(&self) -> bool {
        !self._is_set_low()
    }

    #[inline]
    pub fn is_set_low(&self) -> bool {
        self._is_set_low()
    }

    #[inline]
    pub fn toggle(&mut self) {
        if self._is_set_low() {
            self._set_high()
        } else {
            self._set_low()
        }
    }
}

impl<const P: char, const N: u8, MODE> OutputPin for Pin<P, N, Output<MODE>> {
    type Error = Infallible;

    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<const P: char, const N: u8, MODE> StatefulOutputPin for Pin<P, N, Output<MODE>> {
    #[inline]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_high())
    }

    #[inline]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_low())
    }
}

impl<const P: char, const N: u8, MODE> ToggleableOutputPin for Pin<P, N, Output<MODE>> {
    type Error = Infallible;

    #[inline(always)]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.toggle();
        Ok(())
    }
}

impl<const P: char, const N: u8, MODE> Pin<P, N, Input<MODE>> {
    #[inline]
    pub fn is_high(&self) -> bool {
        !self._is_low()
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        self._is_low()
    }
}

impl<const P: char, const N: u8, MODE> InputPin for Pin<P, N, Input<MODE>> {
    type Error = Infallible;

    #[inline]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    #[inline]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

impl<const P: char, const N: u8> Pin<P, N, Output<OpenDrain>> {
    #[inline]
    pub fn is_high(&self) -> bool {
        !self._is_low()
    }

    #[inline]
    pub fn is_low(&self) -> bool {
        self._is_low()
    }
}

impl<const P: char, const N: u8> InputPin for Pin<P, N, Output<OpenDrain>> {
    type Error = Infallible;

    #[inline]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    #[inline]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

/// Opaque CTL register
pub struct Ctl<const P: char, const H: bool>(());

impl<const P: char, const N: u8, MODE> Pin<P, N, MODE>
where
    MODE: Active,
    Self: HL,
{
    /// Configures the pin to operate as an alternate function push-pull output
    /// pin.
    #[inline]
    pub fn into_alternate_push_pull(
        mut self,
        ctl: &mut <Self as HL>::Ctl,
    ) -> Pin<P, N, Alternate<PushPull>> {
        self.mode::<Alternate<PushPull>>(ctl);
        Pin::new()
    }

    /// Configures the pin to operate as an alternate function open-drain output
    /// pin.
    #[inline]
    pub fn into_alternate_open_drain(
        mut self,
        ctl: &mut <Self as HL>::Ctl,
    ) -> Pin<P, N, Alternate<OpenDrain>> {
        self.mode::<Alternate<OpenDrain>>(ctl);
        Pin::new()
    }

    /// Configures the pin to operate as a floating input pin
    #[inline]
    pub fn into_floating_input(
        mut self,
        ctl: &mut <Self as HL>::Ctl,
    ) -> Pin<P, N, Input<Floating>> {
        self.mode::<Input<Floating>>(ctl);
        Pin::new()
    }

    /// Configures the pin to operate as a pulled down input pin
    #[inline]
    pub fn into_pull_down_input(
        mut self,
        ctl: &mut <Self as HL>::Ctl,
    ) -> Pin<P, N, Input<PullDown>> {
        self.mode::<Input<PullDown>>(ctl);
        Pin::new()
    }

    /// Configures the pin to operate as a pulled up input pin
    #[inline]
    pub fn into_pull_up_input(mut self, ctl: &mut <Self as HL>::Ctl) -> Pin<P, N, Input<PullUp>> {
        self.mode::<Input<PullUp>>(ctl);
        Pin::new()
    }

    /// Configures the pin to operate as an open-drain output pin.
    /// Initial state will be low.
    #[inline]
    pub fn into_open_drain_output(
        self,
        ctl: &mut <Self as HL>::Ctl,
    ) -> Pin<P, N, Output<OpenDrain>> {
        self.into_open_drain_output_with_state(ctl, PinState::Low)
    }

    /// Configures the pin to operate as an open-drain output pin.
    /// `initial_state` specifies whether the pin should be initially high or low.
    #[inline]
    pub fn into_open_drain_output_with_state(
        mut self,
        ctl: &mut <Self as HL>::Ctl,
        initial_state: PinState,
    ) -> Pin<P, N, Output<OpenDrain>> {
        self._set_state(initial_state);
        self.mode::<Output<OpenDrain>>(ctl);
        Pin::new()
    }

    /// Configures the pin to operate as an push-pull output pin.
    /// Initial state will be low.
    #[inline]
    pub fn into_push_pull_output(self, ctl: &mut <Self as HL>::Ctl) -> Pin<P, N, Output<PushPull>> {
        self.into_push_pull_output_with_state(ctl, PinState::Low)
    }

    /// Configures the pin to operate as an push-pull output pin.
    /// `initial_state` specifies whether the pin should be initially high or low.
    #[inline]
    pub fn into_push_pull_output_with_state(
        mut self,
        ctl: &mut <Self as HL>::Ctl,
        initial_state: PinState,
    ) -> Pin<P, N, Output<PushPull>> {
        self._set_state(initial_state);
        self.mode::<Output<PushPull>>(ctl);
        Pin::new()
    }

    /// Configures the pin to operate as an analog input pin
    #[inline]
    pub fn into_analog(mut self, ctl: &mut <Self as HL>::Ctl) -> Pin<P, N, Analog> {
        self.mode::<Analog>(ctl);
        Pin::new()
    }

    /// Configures the pin as a pin that can change between input
    /// and output without changing the type. It starts out
    /// as a floating input
    #[inline]
    pub fn into_dynamic(mut self, ctl: &mut <Self as HL>::Ctl) -> Pin<P, N, Dynamic> {
        self.mode::<Input<Floating>>(ctl);
        Pin::new()
    }
}

// These macros are defined here instead of at the top level in order
// to be able to refer to macro variables from the outer layers.
macro_rules! impl_temp_output {
    ($fn_name:ident, $stateful_fn_name:ident, $mode:ty) => {
        /// Temporarily change the mode of the pin.
        ///
        /// The value of the pin after conversion is undefined. If you
        /// want to control it, use `$stateful_fn_name`
        #[inline]
        pub fn $fn_name(
            &mut self,
            ctl: &mut <Self as HL>::Ctl,
            mut f: impl FnMut(&mut Pin<P, N, $mode>),
        ) {
            self.mode::<$mode>(ctl);
            let mut temp = Pin::<P, N, $mode>::new();
            f(&mut temp);
            self.mode::<$mode>(ctl);
            Self::new();
        }

        /// Temporarily change the mode of the pin.
        ///
        /// Note that the new state is set slightly before conversion
        /// happens. This can cause a short output glitch if switching
        /// between output modes
        #[inline]
        pub fn $stateful_fn_name(
            &mut self,
            ctl: &mut <Self as HL>::Ctl,
            state: PinState,
            mut f: impl FnMut(&mut Pin<P, N, $mode>),
        ) {
            self._set_state(state);
            self.mode::<$mode>(ctl);
            let mut temp = Pin::<P, N, $mode>::new();
            f(&mut temp);
            self.mode::<$mode>(ctl);
            Self::new();
        }
    };
}
macro_rules! impl_temp_input {
    ($fn_name:ident, $mode:ty) => {
        /// Temporarily change the mode of the pin.
        #[inline]
        pub fn $fn_name(
            &mut self,
            ctl: &mut <Self as HL>::Ctl,
            mut f: impl FnMut(&mut Pin<P, N, $mode>),
        ) {
            self.mode::<$mode>(ctl);
            let mut temp = Pin::<P, N, $mode>::new();
            f(&mut temp);
            self.mode::<$mode>(ctl);
            Self::new();
        }
    };
}

impl<const P: char, const N: u8, MODE> Pin<P, N, MODE>
where
    MODE: Active + PinMode,
    Self: HL,
{
    impl_temp_output!(
        as_push_pull_output,
        as_push_pull_output_with_state,
        Output<PushPull>
    );
    impl_temp_output!(
        as_open_drain_output,
        as_open_drain_output_with_state,
        Output<OpenDrain>
    );
    impl_temp_input!(as_floating_input, Input<Floating>);
    impl_temp_input!(as_pull_up_input, Input<PullUp>);
    impl_temp_input!(as_pull_down_input, Input<PullDown>);
}

impl<const P: char, const N: u8, MODE> Pin<P, N, MODE>
where
    Self: HL,
{
    #[inline(always)]
    fn ctl_modify(&mut self, _ctl: &mut <Self as HL>::Ctl, f: impl FnOnce(u32) -> u32) {
        let gpio = unsafe { &(*Gpio::<P>::ptr()) };

        match N {
            0..=7 => {
                gpio.ctl0.modify(|r, w| unsafe { w.bits(f(r.bits())) });
            }
            8..=15 => {
                gpio.ctl1.modify(|r, w| unsafe { w.bits(f(r.bits())) });
            }
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn _set_speed(&mut self, ctl: &mut <Self as HL>::Ctl, speed: Speed) {
        self.ctl_modify(ctl, |r_bits| {
            // TODO
            (r_bits & !(0b11 << Self::OFFSET)) | ((speed as u32) << Self::OFFSET)
        });
    }
}

impl<const P: char, const N: u8, MODE> OutputSpeed for Pin<P, N, Output<MODE>>
where
    Self: HL,
{
    fn set_speed(&mut self, ctl: &mut <Self as HL>::Ctl, speed: Speed) {
        self._set_speed(ctl, speed)
    }
}

impl<const P: char, const N: u8> OutputSpeed for Pin<P, N, Alternate<PushPull>>
where
    Self: HL,
{
    fn set_speed(&mut self, ctl: &mut <Self as HL>::Ctl, speed: Speed) {
        self._set_speed(ctl, speed)
    }
}

// Dynamic pin

impl<const P: char, const N: u8> Pin<P, N, Dynamic>
where
    Self: HL,
{
    #[inline]
    pub fn make_pull_up_input(&mut self, ctl: &mut <Self as HL>::Ctl) {
        // NOTE(unsafe), we have a mutable reference to the current pin
        self.mode::<Input<PullUp>>(ctl);
        self.mode = Dynamic::InputPullUp;
    }

    #[inline]
    pub fn make_pull_down_input(&mut self, ctl: &mut <Self as HL>::Ctl) {
        // NOTE(unsafe), we have a mutable reference to the current pin
        self.mode::<Input<PullDown>>(ctl);
        self.mode = Dynamic::InputPullDown;
    }

    #[inline]
    pub fn make_floating_input(&mut self, ctl: &mut <Self as HL>::Ctl) {
        // NOTE(unsafe), we have a mutable reference to the current pin
        self.mode::<Input<Floating>>(ctl);
        self.mode = Dynamic::InputFloating;
    }

    #[inline]
    pub fn make_push_pull_output(&mut self, ctl: &mut <Self as HL>::Ctl) {
        // NOTE(unsafe), we have a mutable reference to the current pin
        self.mode::<Output<PushPull>>(ctl);
        self.mode = Dynamic::OutputPushPull;
    }

    #[inline]
    pub fn make_open_drain_output(&mut self, ctl: &mut <Self as HL>::Ctl) {
        // NOTE(unsafe), we have a mutable reference to the current pin
        self.mode::<Output<OpenDrain>>(ctl);
        self.mode = Dynamic::OutputOpenDrain;
    }
}

impl PinMode for Input<Floating> {
    const CNF: u32 = 0b01;
    const MODE: u32 = 0b00;
}

impl PinMode for Input<PullDown> {
    const CNF: u32 = 0b10;
    const MODE: u32 = 0b00;
    const PULL: Option<bool> = Some(false);
}

impl PinMode for Input<PullUp> {
    const CNF: u32 = 0b10;
    const MODE: u32 = 0b00;
    const PULL: Option<bool> = Some(true);
}

impl PinMode for Output<OpenDrain> {
    const CNF: u32 = 0b01;
    const MODE: u32 = 0b11;
    const PULL: Option<bool> = Some(true);
}

impl PinMode for Output<PushPull> {
    const CNF: u32 = 0b00;
    const MODE: u32 = 0b11;
}

impl PinMode for Analog {
    const CNF: u32 = 0b00;
    const MODE: u32 = 0b00;
}

impl PinMode for Alternate<PushPull> {
    const CNF: u32 = 0b10;
    const MODE: u32 = 0b11;
}

impl PinMode for Alternate<OpenDrain> {
    const CNF: u32 = 0b11;
    const MODE: u32 = 0b11;
}

impl<const P: char, const N: u8, M> Pin<P, N, M>
where
    Self: HL,
{
    fn mode<MODE: PinMode>(&mut self, ctl: &mut <Self as HL>::Ctl) {
        let gpio = unsafe { &(*Gpio::<P>::ptr()) };

        // Input<PullUp> or Input<PullDown> mode
        if let Some(pullup) = MODE::PULL {
            if pullup {
                gpio.octl
                    .modify(|r, w| unsafe { w.bits(r.bits() | (1 << N)) });
            } else {
                gpio.octl
                    .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << N)) });
            }
        }

        let bits = (MODE::CNF << 2) | MODE::MODE;

        self.ctl_modify(ctl, |r_bits| {
            (r_bits & !(0b1111 << Self::OFFSET)) | (bits << Self::OFFSET)
        });
    }
}

gpio!(GPIOA, gpioa, PAx, 'A', [
    PA0: (pa0, 0),
    PA1: (pa1, 1),
    PA2: (pa2, 2),
    PA3: (pa3, 3),
    PA4: (pa4, 4),
    PA5: (pa5, 5),
    PA6: (pa6, 6),
    PA7: (pa7, 7),
    PA8: (pa8, 8),
    PA9: (pa9, 9),
    PA10: (pa10, 10),
    PA11: (pa11, 11),
    PA12: (pa12, 12),
    PA13: (pa13, 13, Debugger),
    PA14: (pa14, 14, Debugger),
    PA15: (pa15, 15),
]);

gpio!(GPIOB, gpiob, PBx, 'B', [
    PB0: (pb0, 0),
    PB1: (pb1, 1),
    PB2: (pb2, 2),
    PB3: (pb3, 3),
    PB4: (pb4, 4),
    PB5: (pb5, 5),
    PB6: (pb6, 6),
    PB7: (pb7, 7),
    PB8: (pb8, 8),
    PB9: (pb9, 9),
    PB10: (pb10, 10),
    PB11: (pb11, 11),
    PB12: (pb12, 12),
    PB13: (pb13, 13),
    PB14: (pb14, 14),
    PB15: (pb15, 15),
]);

gpio!(GPIOC, gpioc, PCx, 'C', [
    PC0: (pc0, 0),
    PC1: (pc1, 1),
    PC2: (pc2, 2),
    PC3: (pc3, 3),
    PC4: (pc4, 4),
    PC5: (pc5, 5),
    PC6: (pc6, 6),
    PC7: (pc7, 7),
    PC8: (pc8, 8),
    PC9: (pc9, 9),
    PC10: (pc10, 10),
    PC11: (pc11, 11),
    PC12: (pc12, 12),
    PC13: (pc13, 13),
    PC14: (pc14, 14),
    PC15: (pc15, 15),
]);

gpio!(GPIOD, gpiod, PDx, 'D', [
    PD0: (pd0, 0),
    PD1: (pd1, 1),
    PD2: (pd2, 2),
    PD3: (pd3, 3),
    PD4: (pd4, 4),
    PD5: (pd5, 5),
    PD6: (pd6, 6),
    PD7: (pd7, 7),
    PD8: (pd8, 8),
    PD9: (pd9, 9),
    PD10: (pd10, 10),
    PD11: (pd11, 11),
    PD12: (pd12, 12),
    PD13: (pd13, 13),
    PD14: (pd14, 14),
    PD15: (pd15, 15),
]);

struct Gpio<const P: char>;
impl<const P: char> Gpio<P> {
    const fn ptr() -> *const crate::pac::gpioa::RegisterBlock {
        match P {
            'A' => crate::pac::GPIOA::ptr(),
            'B' => crate::pac::GPIOB::ptr() as _,
            'C' => crate::pac::GPIOC::ptr() as _,
            'D' => crate::pac::GPIOD::ptr() as _,
            _ => unreachable!(),
        }
    }
}
