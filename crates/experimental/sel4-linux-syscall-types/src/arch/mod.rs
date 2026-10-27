//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

cfg_select! {
    target_arch = "aarch64" => {
        mod aarch64;
        pub use aarch64::*;
    }
    target_arch = "arm" => {
        mod arm;
        pub use arm::*;
    }
    target_arch = "riscv64" => {
        mod riscv64;
        pub use riscv64::*;
    }
    target_arch = "riscv32" => {
        mod riscv32;
        pub use riscv32::*;
    }
    target_arch = "x86_64" => {
        mod x86_64;
        pub use x86_64::*;
    }
    _ => {
        compile_error!("unsupported architecture");
    }
}
