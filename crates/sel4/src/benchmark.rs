//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use sel4_config::sel4_cfg_if;

use crate::{sys, Error, Result, cap, Word};

pub fn benchmark_reset_log() -> Result<()> {
    Error::wrap(sys::seL4_BenchmarkResetLog())
}

pub fn benchmark_finalize_log() -> Word {
    sys::seL4_BenchmarkFinalizeLog()
}

pub fn benchmark_set_log_buffer(frame: cap::UnspecifiedPage) -> Result<()> {
    Error::wrap(sys::seL4_BenchmarkSetLogBuffer(frame.bits()))
}

sel4_cfg_if! {
    if #[sel4_cfg(BENCHMARK_TRACK_UTILISATION)] {
        pub fn benchmark_get_thread_utilisation(tcb: cap::Tcb) {
            sys::seL4_BenchmarkGetThreadUtilisation(tcb.bits())
        }

        pub fn benchmark_reset_thread_utilisation(tcb: cap::Tcb) {
            sys::seL4_BenchmarkResetThreadUtilisation(tcb.bits())
        }

        sel4_cfg_if! {
            if #[sel4_cfg(DEBUG_BUILD)] {
                pub fn benchmark_dump_all_thread_utilisation() {
                    sys::seL4_BenchmarkDumpAllThreadsUtilisation()
                }

                pub fn benchmark_reset_all_thread_utilisation() {
                    sys::seL4_BenchmarkResetAllThreadsUtilisation()
                }
            }
        }
    }
}
