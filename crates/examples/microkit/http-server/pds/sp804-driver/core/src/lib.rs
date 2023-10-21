//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::time::Duration;

use tock_registers::interfaces::ReadWriteable;

mod device;

use device::{Control, Device, Timer};

pub struct Driver {
    device: Device,
    freq: u64, // Hz
    high_bits: u32,
    most_recent_value: u32,
}

impl Driver {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn new(ptr: *mut (), freq: u64) -> Self {
        let mut this = Self {
            device: Device::new(ptr),
            freq,
            high_bits: 0,
            most_recent_value: !0,
        };
        this.init();
        this
    }

    fn init(&mut self) {
        let control_common =
            Control::TimerEn::Disabled + Control::TimerPre::Div256 + Control::TimerSize::Use32Bit;

        self.timer_for_reading()
            .control()
            .modify(control_common + Control::IntEnable::Enabled);
        self.timer_for_reading().set_free_running_mode();
        self.timer_for_reading().clear_interrupt();
        self.timer_for_reading().set_load(!0);

        self.timer_for_writing()
            .control()
            .modify(control_common + Control::IntEnable::Disabled);
        self.timer_for_writing().set_one_shot_mode();
        self.timer_for_writing().clear_interrupt();
        self.timer_for_writing().set_load(0);

        self.timer_for_reading()
            .control()
            .modify(Control::TimerEn::Enabled);
    }

    fn scaled_freq(&self) -> u64 {
        self.freq / 256
    }

    fn timer_for_reading(&self) -> &Timer {
        self.device.timer_1()
    }

    fn timer_for_writing(&self) -> &Timer {
        self.device.timer_2()
    }

    fn current_value_checking_for_overflow(&mut self) -> u32 {
        let value = self.timer_for_reading().current_value();
        if value > self.most_recent_value {
            self.high_bits += 1;
        }
        self.most_recent_value = value;
        value
    }

    fn check_for_overflow(&mut self) {
        let _ = self.current_value_checking_for_overflow();
    }

    pub fn now(&mut self) -> Duration {
        let value = self.current_value_checking_for_overflow();
        let ticks = ((u64::from(self.high_bits) + 1) << 32) - u64::from(value);
        self.ticks_to_duration(ticks)
    }

    fn ticks_to_duration(&self, ticks: u64) -> Duration {
        Duration::from_nanos(
            u64::try_from((u128::from(ticks) * 1_000_000_000) / u128::from(self.scaled_freq()))
                .unwrap(),
        )
    }

    fn duration_to_ticks(&self, d: Duration) -> u128 {
        (d.as_nanos() * u128::from(self.scaled_freq())) / 1_000_000_000
    }

    pub fn handle_interrupt(&mut self) {
        if self.timer_for_reading().masked_interrupt_status() {
            self.check_for_overflow();
            self.timer_for_reading().clear_interrupt();
        }
        if self.timer_for_writing().masked_interrupt_status() {
            self.clear_timeout();
            self.timer_for_writing().clear_interrupt();
        }
    }

    pub fn set_timeout(&self, relative: Duration) {
        self.clear_timeout();
        self.timer_for_writing()
            .set_load(self.duration_to_ticks(relative).try_into().unwrap());
        self.timer_for_writing()
            .control()
            .modify(Control::TimerEn::Enabled + Control::IntEnable::Enabled);
    }

    pub fn clear_timeout(&self) {
        self.timer_for_writing()
            .control()
            .modify(Control::TimerEn::Disabled + Control::IntEnable::Disabled);
        self.timer_for_writing().set_load(0);
    }
}
