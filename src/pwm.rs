use crate::gpio::{
    gpioa::{PA10, PA11, PA8, PA9},
    Alternate, AF2,
};
use crate::pac::{timer0, TIMER0};
use crate::time::Hertz;
use crate::time::U32Ext;
use crate::timer::{Timer, TimerExt};
use core::marker::{Copy, PhantomData};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Channel {
    C0,
    C1,
    C2,
    C3,
}

trait TimerRegExt {
    fn disable_channel(&self, channel: Channel);
    fn enable_channel(&self, channel: Channel);
    fn get_duty(&self, channel: Channel) -> u16;
    fn set_duty(&self, channel: Channel, duty: u16);
    fn get_max_duty(&self) -> u16;
}

pub struct Pwm<TIMER, PINS> {
    clock: Hertz,
    pins: PINS,
    timer: TIMER,
}

pub struct PwmChannel<TIMER, PIN> {
    channel: Channel,
    _pin: PIN,
    _timer: PhantomData<TIMER>,
}

pub struct Ch0;
pub struct Ch1;
pub struct Ch2;
pub struct Ch3;

pub trait Pin<TIMER, CHANNEL> {}

pub trait Pins<TIMER> {
    fn uses_channel(&self, channel: Channel) -> bool;
}

impl Pin<TIMER0, Ch0> for PA8<Alternate<AF2>> {}
impl Pin<TIMER0, Ch1> for PA9<Alternate<AF2>> {}
impl Pin<TIMER0, Ch2> for PA10<Alternate<AF2>> {}
impl Pin<TIMER0, Ch3> for PA11<Alternate<AF2>> {}

impl<P0, P1, P2, P3, TIMER> Pins<TIMER> for (Option<P0>, Option<P1>, Option<P2>, Option<P3>)
where
    P0: Pin<TIMER, Ch0>,
    P1: Pin<TIMER, Ch1>,
    P2: Pin<TIMER, Ch2>,
    P3: Pin<TIMER, Ch3>,
{
    fn uses_channel(&self, channel: Channel) -> bool {
        match channel {
            Channel::C0 => self.0.is_some(),
            Channel::C1 => self.1.is_some(),
            Channel::C2 => self.2.is_some(),
            Channel::C3 => self.3.is_some(),
        }
    }
}

impl Timer<TIMER0> {
    pub fn pwm<PINS, T>(self, pins: PINS, freq: T) -> Pwm<TIMER0, PINS>
    where
        PINS: Pins<TIMER0>,
        T: Into<Hertz>,
    {
        // TIMER0 has a break function that deactivates the outputs. This bit automatically activates
        // the output when no break input is present.
        self.timer.cchp.modify(|_, w| w.oaen().automatic());

        let Self { timer, clock } = self;
        timer0(timer, pins, freq.into(), clock)
    }
}

fn timer0<PINS>(mut timer: TIMER0, pins: PINS, freq: Hertz, clock: Hertz) -> Pwm<TIMER0, PINS>
where
    PINS: Pins<TIMER0>,
{
    if pins.uses_channel(Channel::C0) {
        timer
            .chctl0_output()
            .modify(|_, w| w.ch0comsen().set_bit().ch0comctl().pwm_mode1());
    }
    if pins.uses_channel(Channel::C1) {
        timer
            .chctl0_output()
            .modify(|_, w| w.ch1comsen().set_bit().ch1comctl().pwm_mode1());
    }
    if pins.uses_channel(Channel::C2) {
        timer
            .chctl1_output()
            .modify(|_, w| w.ch2comsen().set_bit().ch2comctl().pwm_mode1());
    }
    if pins.uses_channel(Channel::C3) {
        timer
            .chctl1_output()
            .modify(|_, w| w.ch3comsen().set_bit().ch3comctl().pwm_mode1());
    }
    timer.configure_prescaler_reload(freq, clock);
    // Trigger an update event to load the prescaler value to the clock
    timer.reset_counter();

    timer.ctl0.write(|w| {
        w.cam()
            .edge_aligned()
            .dir()
            .up()
            .spm()
            .disabled()
            .cen()
            .enabled()
    });

    Pwm {
        clock: clock,
        pins,
        timer,
    }
}

impl<P0, P1, P2, P3, TIMER> Pwm<TIMER, (Option<P0>, Option<P1>, Option<P2>, Option<P3>)> {
    /// Split the timer into separate PWM channels.
    pub fn split(
        self,
    ) -> (
        Option<PwmChannel<TIMER, P0>>,
        Option<PwmChannel<TIMER, P1>>,
        Option<PwmChannel<TIMER, P2>>,
        Option<PwmChannel<TIMER, P3>>,
    ) {
        (
            self.pins.0.map(|pin| PwmChannel {
                channel: Channel::C0,
                _pin: pin,
                _timer: PhantomData,
            }),
            self.pins.1.map(|pin| PwmChannel {
                channel: Channel::C1,
                _pin: pin,
                _timer: PhantomData,
            }),
            self.pins.2.map(|pin| PwmChannel {
                channel: Channel::C2,
                _pin: pin,
                _timer: PhantomData,
            }),
            self.pins.3.map(|pin| PwmChannel {
                channel: Channel::C3,
                _pin: pin,
                _timer: PhantomData,
            }),
        )
    }
}

impl<PINS> Pwm<TIMER0, PINS>
where
    PINS: Pins<TIMER0>,
{
    /// Stop the timer and release it and the pins to be used for something else.
    pub fn stop(self) -> (Timer<TIMER0>, PINS) {
        self.timer.chctl2.reset();
        self.timer.chctl0_output().reset();
        self.timer.chctl1_output().reset();
        self.timer.ch0cv.reset();
        self.timer.ch1cv.reset();
        self.timer.ch2cv.reset();
        self.timer.ch3cv.reset();
        self.timer.cchp.reset();
        (
            Timer {
                timer: self.timer,
                clock: self.clock,
            },
            self.pins,
        )
    }
}

impl<PIN> embedded_hal::PwmPin for PwmChannel<TIMER0, PIN> {
    type Duty = u16;

    fn disable(&mut self) {
        unsafe { &*TIMER0::ptr() }.disable_channel(self.channel);
    }

    fn enable(&mut self) {
        unsafe { &*TIMER0::ptr() }.enable_channel(self.channel);
    }

    fn get_duty(&self) -> u16 {
        unsafe { &*TIMER0::ptr() }.get_duty(self.channel)
    }

    fn get_max_duty(&self) -> u16 {
        unsafe { &*TIMER0::ptr() }.get_max_duty()
    }

    fn set_duty(&mut self, duty: u16) {
        unsafe { &*TIMER0::ptr() }.set_duty(self.channel, duty);
    }
}

impl TimerRegExt for timer0::RegisterBlock {
    fn disable_channel(&self, channel: Channel) {
        match channel {
            Channel::C0 => self.chctl2.modify(|_, w| w.ch0en().disabled()),
            Channel::C1 => self.chctl2.modify(|_, w| w.ch1en().disabled()),
            Channel::C2 => self.chctl2.modify(|_, w| w.ch2en().disabled()),
            Channel::C3 => self.chctl2.modify(|_, w| w.ch3en().disabled()),
        }
    }

    fn enable_channel(&self, channel: Channel) {
        match channel {
            Channel::C0 => self.chctl2.modify(|_, w| w.ch0en().enabled()),
            Channel::C1 => self.chctl2.modify(|_, w| w.ch1en().enabled()),
            Channel::C2 => self.chctl2.modify(|_, w| w.ch2en().enabled()),
            Channel::C3 => self.chctl2.modify(|_, w| w.ch3en().enabled()),
        }
    }

    fn get_duty(&self, channel: Channel) -> u16 {
        match channel {
            Channel::C0 => self.ch0cv.read().ch0val().bits(),
            Channel::C1 => self.ch1cv.read().ch1val().bits(),
            Channel::C2 => self.ch2cv.read().ch2val().bits(),
            Channel::C3 => self.ch3cv.read().ch3val().bits(),
        }
    }

    fn set_duty(&self, channel: Channel, duty: u16) {
        match channel {
            Channel::C0 => self.ch0cv.write(|w| w.ch0val().bits(duty)),
            Channel::C1 => self.ch1cv.write(|w| w.ch1val().bits(duty)),
            Channel::C2 => self.ch2cv.write(|w| w.ch2val().bits(duty)),
            Channel::C3 => self.ch3cv.write(|w| w.ch3val().bits(duty)),
        }
    }

    fn get_max_duty(&self) -> u16 {
        self.car.read().car().bits()
    }
}

impl<PINS> embedded_hal::Pwm for Pwm<TIMER0, PINS>
where
    PINS: Pins<TIMER0>,
{
    type Channel = Channel;
    type Duty = u16;
    type Time = Hertz;

    fn disable(&mut self, channel: Self::Channel) {
        assert!(self.pins.uses_channel(channel));
        self.timer.disable_channel(channel);
    }

    fn enable(&mut self, channel: Self::Channel) {
        assert!(self.pins.uses_channel(channel));
        self.timer.enable_channel(channel);
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        assert!(self.pins.uses_channel(channel));
        self.timer.get_duty(channel)
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        assert!(self.pins.uses_channel(channel));
        self.timer.set_duty(channel, duty);
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.timer.get_max_duty()
    }

    fn get_period(&self) -> Self::Time {
        let presaler: u32 = self.timer.psc.read().psc().bits().into();
        let auto_reload_value: u32 = self.timer.car.read().car().bits().into();

        // Length in ms of an internal clock pulse
        (self.clock.0 / (presaler * auto_reload_value)).hz()
    }

    fn set_period<T>(&mut self, period: T)
    where
        T: Into<Self::Time>,
    {
        self.timer
            .configure_prescaler_reload(period.into(), self.clock);
    }
}
