//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(clippy::write_with_newline)]

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use core::fmt;

use addr2line::fallible_iterator::FallibleIterator;
use addr2line::gimli::read::Reader;
use addr2line::gimli::Error;
use addr2line::Context;

pub use addr2line::object::File;

use crate::Backtrace;

// TODO handle inlining better (see TODOs scattered throughout this file)

impl<T> Backtrace<T> {
    pub fn symbolize<R: Reader>(
        &self,
        ctx: &Context<R>,
        w: &mut impl fmt::Write,
    ) -> Result<(), Error> {
        if let Some(ref err) = self.postamble.error {
            writeln!(w, "    error: {:?}", err).unwrap();
        }
        for (i, entry) in self.entries.iter().enumerate() {
            let mut first = true;
            let frame = &entry.stack_frame;
            // TODO
            // let mut seen = false;
            // let initial_location = ctx.find_location(frame.ip as u64)?;
            ctx.find_frames(frame.ip as u64)
                .skip_all_loads()?
                .for_each(|inner_frame| {
                    if first {
                        write!(w, " {:4}:  {:#18x} - ", i, frame.ip).unwrap();
                    } else {
                        write!(w, " {:4}   {:18  }   ", "", "").unwrap();
                    }
                    // TODO
                    // if inner_frame.location == frame {
                    //     seen = true;
                    // }
                    match inner_frame.function {
                        Some(f) => {
                            // TODO
                            // let raw_name = f.raw_name()?;
                            // let demangled = demangle(&raw_name);
                            let demangled = f.demangle()?;
                            write!(w, "{}", demangled).unwrap()
                        }
                        None => write!(w, "<unknown>").unwrap(),
                    }
                    write!(w, "\n").unwrap();
                    // TODO
                    // if let Some(loc) = inner_frame.location {
                    //     writeln!(w, "      {:18}       at {}", "", fmt_location(loc)).unwrap();
                    // }
                    first = false;
                    Ok(())
                })?;
            // TODO this isn't accurate
            if let Some(loc) = ctx.find_location(frame.ip as u64)? {
                writeln!(w, "      {:18}       at {}", "", fmt_location(loc)).unwrap();
            }
            // TODO
            // if !seen {
            //     write!(w, "      ").unwrap();
            //     write!(w, "warning: initial location missing: {}", initial_location).unwrap();
            //     write!(w, "\n").unwrap();
            // }
        }
        Ok(())
    }
}

fn fmt_location(loc: addr2line::Location) -> String {
    format!(
        "{} {},{}",
        loc.file.unwrap_or("<unknown>"),
        loc.line
            .map(|x| x.to_string())
            .unwrap_or(String::from("<unknown>")),
        loc.column
            .map(|x| x.to_string())
            .unwrap_or(String::from("<unknown>")),
    )
}
