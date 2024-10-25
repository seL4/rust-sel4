//
// Copyright 2024, Colias Group, LLC
// Copyright (c) 2016-2018 The gimli Developers
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

// Adapted from:
// https://github.com/gimli-rs/addr2line/blob/0.24.2/src/bin/addr2line.rs

#![no_std]

extern crate alloc;

use alloc::borrow::Cow;
use core::fmt;

use addr2line::fallible_iterator::FallibleIterator;
use addr2line::gimli;
use addr2line::{Context, Location};

fn print_loc(
    w: &mut impl fmt::Write,
    loc: Option<&Location<'_>>,
    basenames: bool,
) -> Result<(), fmt::Error> {
    if let Some(loc) = loc {
        if let Some(ref file) = loc.file.as_ref() {
            let path = if basenames {
                file
            } else {
                // TODO
                file
            };
            write!(w, "{}:", path)?;
        } else {
            write!(w, "??:")?;
        }
        if let Some(line) = loc.line {
            write!(w, "{}", line)?;
        } else {
            write!(w, "?")?;
        }
        writeln!(w)?;
    } else {
        writeln!(w, "??:0")?;
    }
    Ok(())
}

fn print_function(
    w: &mut impl fmt::Write,
    name: Option<&str>,
    language: Option<gimli::DwLang>,
    demangle: bool,
) -> Result<(), fmt::Error> {
    if let Some(name) = name {
        if demangle {
            write!(w, "{}", addr2line::demangle_auto(Cow::from(name), language))?;
        } else {
            write!(w, "{}", name)?;
        }
    } else {
        write!(w, "??")?;
    }
    Ok(())
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Options {
    pub do_functions: bool,
    pub do_inlines: bool,
    pub print_addrs: bool,
    pub basenames: bool,
    pub demangle: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            do_functions: true,
            do_inlines: true,
            print_addrs: true,
            basenames: true,
            demangle: true,
        }
    }
}

pub fn symbolize(
    w: &mut impl fmt::Write,
    ctx: &Context<impl gimli::Reader>,
    opts: &Options,
    addrs: impl Iterator<Item = u64>,
) -> Result<(), fmt::Error> {
    for probe in addrs {
        if opts.print_addrs {
            let addr = probe;
            write!(w, "0x{:016x}: ", addr)?;
        }

        if opts.do_functions || opts.do_inlines {
            let mut printed_anything = false;
            let mut frames = ctx.find_frames(probe).skip_all_loads().unwrap().peekable();
            let mut first = true;
            while let Some(frame) = frames.next().unwrap() {
                if !first {
                    write!(w, " (inlined by) ")?;
                }
                first = false;

                if opts.do_functions {
                    // TODO
                    // See:
                    // https://github.com/gimli-rs/addr2line/blob/621a3abe985b32f43dd1e8c10e003abe902c68e2/src/bin/addr2line.rs#L223-L242
                    if let Some(func) = frame.function {
                        print_function(
                            w,
                            func.raw_name().ok().as_deref(),
                            func.language,
                            opts.demangle,
                        )?;
                    } else {
                        print_function(w, None, None, opts.demangle)?;
                    }

                    write!(w, " at ")?;
                }

                print_loc(w, frame.location.as_ref(), opts.basenames)?;

                printed_anything = true;

                if !opts.do_inlines {
                    break;
                }
            }

            if !printed_anything {
                if opts.do_functions {
                    // TODO
                    // let name = ctx.find_symbol(probe);
                    let name = None;

                    print_function(w, name, None, opts.demangle)?;

                    write!(w, " at ")?;
                }

                print_loc(w, None, opts.basenames)?;
            }
        } else {
            let loc = ctx.find_location(probe).unwrap();
            print_loc(w, loc.as_ref(), opts.basenames)?;
        }
    }

    Ok(())
}
