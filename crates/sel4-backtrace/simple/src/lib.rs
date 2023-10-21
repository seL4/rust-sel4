//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(never_type)]
#![feature(unwrap_infallible)]

use sel4_backtrace::{BacktraceSendWithToken, BacktraceSendWithoutToken};
use sel4_panicking_env::{debug_print, debug_println};

#[cfg(feature = "alloc")]
use sel4_backtrace::Backtrace;

// TODO
// Improve flexibility by adding lifetime logic to upstream traits.
pub struct SimpleBacktracing(SimpleBacktraceSend);

impl SimpleBacktracing {
    pub fn new(image: Option<&'static str>) -> Self {
        Self(SimpleBacktraceSend::new(image))
    }
}

impl SimpleBacktracing {
    pub fn collect_and_send(&self) {
        debug_println!("collecting and sending stack backtrace");
        debug_print!("    ");
        let r = self.0.collect_and_send().into_ok();
        debug_println!();
        debug_println!();
        if r.is_err() {
            debug_println!("error encountered while collecting and sending stack backtrace");
        }
    }

    #[cfg(feature = "alloc")]
    pub fn collect(&self) -> Backtrace<Option<&'static str>> {
        debug_println!("collecting stack backtrace");
        self.0.collect()
    }

    #[cfg(feature = "alloc")]
    pub fn send(&self, bt: &Backtrace<Option<&'static str>>) {
        debug_println!("sending stack backtrace");
        debug_print!("    ");
        let r = self.0.send(bt).into_ok();
        debug_println!();
        debug_println!();
        if r.is_err() {
            debug_println!("error encountered while sending stack backtrace");
        }
    }
}

struct SimpleBacktraceSend {
    image: Option<&'static str>,
}

impl SimpleBacktraceSend {
    pub fn new(image: Option<&'static str>) -> Self {
        Self { image }
    }
}

impl BacktraceSendWithoutToken for SimpleBacktraceSend {
    type Image = Option<&'static str>;
    type TxError = !;

    fn image(&self) -> Self::Image {
        self.image
    }

    fn send_byte(&self, byte: u8) -> Result<(), Self::TxError> {
        debug_print!("{:02x}", byte);
        Ok(())
    }
}
