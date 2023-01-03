// Copyright 2023 The gd32e103-hal authors.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # HAL for the GD32E103 family of microcontrollers
//!
//! This is an implementation of the [`embedded-hal`] traits for the GD32E103 family of
//! microcontrollers.
//!
//! [`embedded-hal`]: https://crates.io/crates/embedded-hal
//!
//! # Usage
//!
//! ## Building an application (binary crate)
//!
//! A detailed usage guide can be found in the [README]
//!
//! ## Commonly used setup
//! Almost all peripherals require references to some registers in `RCU`. The following
//! code shows how to set up those registers
//!
//! ```rust
//! // Get access to the device specific peripherals from the peripheral access crate
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Take ownership over the raw RCU and FMC devices and convert tem into the corresponding HAL
//! //  structs.
//! let mut rcu = dp.RCU.constrain();
//! let mut flash = p.FMC.constrain();
//!
//! // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
//! // `clocks`
//! let clocks = rcu.cfgr.freeze(&mut flash.ws);
//! ```
//!
//! ## Usage examples
//!
//! See the [examples] folder.
//!
//! Most of the examples require the following additional dependencies
//! ```toml
//! [dependencies]
//! embedded-hal = "0.2.3"
//! nb = "0.1.2"
//! cortex-m = "0.6.2"
//! cortex-m-rt = "0.6.11"
//! # Panic behaviour, see https://crates.io/keywords/panic-impl for alternatives
//! panic-halt = "0.2.0"
//! ```
//!
//! [examples]: https://github.com/gd32-rust/gd32e103-hal/tree/main/examples
//! [README]: https://github.com/gd32-rust/gd32e103-hal

#![no_std]
#![deny(broken_intra_doc_links)]

pub use gd32e1::gd32e103 as pac;

pub mod adc;
//pub mod backup_domain;
//pub mod can;
pub mod crc;
pub mod delay;
pub mod dma;
pub mod flash;
pub mod gpio;
pub mod i2c;
pub mod prelude;
pub mod pwm;
//pub mod pwm_input;
pub mod rcu;
//pub mod rtc;
pub mod serial;
//pub mod spi;
pub mod time;
pub mod timer;
pub mod watchdog;
