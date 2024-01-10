//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::convert::Infallible;

#[cfg(feature = "postcard")]
use serde::Serialize;

use sel4_backtrace_types::{Entry, Error as BacktraceError};

cfg_if::cfg_if! {
    if #[cfg(feature = "unwinding")] {
        use core::ffi::c_void;

        use unwinding::abi::*;

        use sel4_backtrace_types::StackFrame;

        pub fn collect_with<F: FnMut(Entry) -> Result<(), E>, E>(f: F) -> Option<BacktraceError> {
            struct CallbackData<F1> {
                f: F1,
            }

            extern "C" fn callback<F1: FnMut(Entry) -> Result<(), E1>, E1>(
                unwind_ctx: &mut UnwindContext,
                arg: *mut c_void,
            ) -> UnwindReasonCode {
                let data = unsafe { &mut *(arg as *mut CallbackData<F1>) };
                let ip = _Unwind_GetIP(unwind_ctx);
                match (data.f)(Entry {
                    stack_frame: StackFrame {
                        ip,
                    },
                }) {
                    Ok(()) => UnwindReasonCode::NO_REASON,
                    Err(_) => UnwindReasonCode::FATAL_PHASE2_ERROR,
                }
            }

            let mut data = CallbackData { f };
            let code = _Unwind_Backtrace(callback::<F, E>, &mut data as *mut _ as _);
            match code {
                UnwindReasonCode::END_OF_STACK => None,
                _ => Some(BacktraceError {
                    unwind_reason_code: code.0,
                }),
            }
        }
    } else {
        pub fn collect_with<F: FnMut(Entry) -> Result<(), E>, E>(_f: F) -> Option<BacktraceError> {
            None
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        extern crate alloc;

        pub use sel4_backtrace_types::Backtrace;

        pub fn collect<T>(image: T) -> Backtrace<T> {
            let mut builder = Backtrace::builder(image);
            let error = collect_with(|entry| {
                builder.append(entry);
                Ok::<_, Infallible>(())
            });
            builder.finalize(error)
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "postcard")] {
                pub fn send_with<T: Serialize, F: FnMut(u8) -> Result<(), E>, E>(
                    bt: &Backtrace<T>,
                    mut send_byte: F,
                ) -> postcard::Result<()> {
                    bt.preamble.send(&mut send_byte)?;
                    for entry in bt.entries.iter() {
                        entry.send(&mut send_byte)?;
                    }
                    bt.postamble.send(&mut send_byte)?;
                    Ok(())
                }
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "postcard")] {
        use sel4_backtrace_types::{Postamble, Preamble};

        pub fn collect_and_send_with<T: Serialize, F: FnMut(u8) -> Result<(), E>, E>(
            image: T,
            mut send_byte: F,
        ) -> postcard::Result<()> {
            let preamble = Preamble {
                image,
            };
            preamble.send(&mut send_byte)?;
            let error = collect_with(|entry| entry.send(&mut send_byte).map_err(|_| ()));
            let postamble = Postamble { error };
            postamble.send(&mut send_byte)?;
            Ok(())
        }

        pub trait BacktraceSendWithoutToken {
            type Image: Serialize;
            type TxError;

            fn image(&self) -> Self::Image;

            fn send_byte(&self, byte: u8) -> Result<(), Self::TxError>;
        }

        impl<T: BacktraceSendWithoutToken> BacktraceSendWithToken for T {
            type Image = <Self as BacktraceSendWithoutToken>::Image;
            type Token = ();
            type ControlError = Infallible;
            type TxError = <Self as BacktraceSendWithoutToken>::TxError;

            fn image(&self) -> Self::Image {
                <Self as BacktraceSendWithoutToken>::image(self)
            }

            fn start(&self) -> Result<Self::Token, Self::ControlError> {
                Ok(())
            }

            fn send_byte(&self, _token: &Self::Token, byte: u8) -> Result<(), Self::TxError> {
                <Self as BacktraceSendWithoutToken>::send_byte(self, byte)
            }

            fn finish(&self, _token: Self::Token) -> Result<(), Self::ControlError> {
                Ok(())
            }
        }

        pub trait BacktraceSendWithToken {
            type Image: Serialize;
            type Token;
            type ControlError;
            type TxError;

            fn image(&self) -> Self::Image;

            fn start(&self) -> Result<Self::Token, Self::ControlError>;

            fn send_byte(&self, token: &Self::Token, byte: u8) -> Result<(), Self::TxError>;

            fn finish(&self, token: Self::Token) -> Result<(), Self::ControlError>;

            fn collect_and_send(&self) -> Result<Result<(), postcard::Error>, Self::ControlError> {
                let token = self.start()?;
                let r = collect_and_send_with(self.image(), |b| {
                    self.send_byte(&token, b)
                });
                self.finish(token)?;
                Ok(r)
            }

            cfg_if::cfg_if! {
                if #[cfg(feature = "alloc")] {
                    fn collect(&self) -> Backtrace<Self::Image> {
                        collect(self.image())
                    }

                    fn send(
                        &self,
                        bt: &Backtrace<Self::Image>,
                    ) -> Result<Result<(), postcard::Error>, Self::ControlError> {
                        let token = self.start()?;
                        let r = send_with(bt, |b| {
                            self.send_byte(&token, b)
                        });
                        self.finish(token)?;
                        Ok(r)
                    }
                }
            }
        }
    }
}
