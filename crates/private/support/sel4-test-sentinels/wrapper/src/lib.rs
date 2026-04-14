//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::io;
use std::io::{Read, Write};
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Error, bail};

pub struct Sentinels<T> {
    pub sequences: Vec<Sequence<T>>,
}

pub struct Sequence<T> {
    pub bytes: Vec<u8>,
    pub contiguous: bool,
    pub suppress: SuppressFn,
    pub value: T,
}

type SuppressFn = Box<dyn Fn(&[u8], usize) -> bool>;

struct Observer<'a, T> {
    sentinels: &'a Sentinels<T>,
    states: Vec<usize>,
}

impl<'a, T> Observer<'a, T> {
    fn new(sentinels: &'a Sentinels<T>) -> Self {
        let n = sentinels.sequences.len();
        Self {
            sentinels,
            states: vec![0; n],
        }
    }

    fn observe(&mut self, b: u8) -> (Option<&'a T>, bool) {
        let mut suppress = false;
        let value_opt = self
            .sentinels
            .sequences
            .iter()
            .zip(self.states.iter_mut())
            .find_map(|(sequence, i)| {
                if b == sequence.bytes[*i] {
                    suppress |= (sequence.suppress)(&sequence.bytes, *i);
                    *i += 1;
                    if *i == sequence.bytes.len() {
                        return Some(&sequence.value);
                    }
                } else if sequence.contiguous {
                    *i = 0;
                }
                None
            });
        (value_opt, suppress)
    }
}

pub fn default_sentinels() -> Sentinels<bool> {
    Sentinels {
        sequences: vec![
            Sequence {
                bytes: b"INDICATE_SUCCESS\x06".to_vec(),
                contiguous: false,
                suppress: Box::new(suppress_last),
                value: true,
            },
            Sequence {
                bytes: b"INDICATE_FAILURE\x15".to_vec(),
                contiguous: false,
                suppress: Box::new(suppress_last),
                value: false,
            },
            Sequence {
                bytes: b"TEST_PASS".to_vec(),
                contiguous: true,
                suppress: Box::new(never_suppress),
                value: true,
            },
            Sequence {
                bytes: b"TEST_FAIL".to_vec(),
                contiguous: true,
                suppress: Box::new(never_suppress),
                value: false,
            },
        ],
    }
}

fn suppress_last(sequence_bytes: &[u8], i: usize) -> bool {
    i == sequence_bytes.len() - 1
}

fn never_suppress(_sequence_bytes: &[u8], _i: usize) -> bool {
    false
}

#[derive(Debug)]
pub enum WrapperResult<T> {
    Sentinel(T),
    Exit(ExitStatus),
}

impl WrapperResult<&bool> {
    pub fn success_ok(&self) -> Result<(), Error> {
        match self {
            Self::Sentinel(false) => bail!("failure via sentinel"),
            Self::Exit(c) if !c.success() => bail!(
                "failure via exit code (code: {})",
                c.code()
                    .map(|i| i.to_string())
                    .unwrap_or("unknown".to_owned())
            ),
            _ => Ok(()),
        }
    }
}

impl<T> Sentinels<T> {
    pub fn wrap(&self, mut cmd: Command) -> Result<WrapperResult<&T>, Error> {
        let mut observer = Observer::new(self);

        let mut child = cmd.stdin(Stdio::null()).stdout(Stdio::piped()).spawn()?;
        let mut child_stdout = child.stdout.take().unwrap();
        let mut stdout = io::stdout().lock();

        loop {
            let mut buf = [0u8; 1];

            match child_stdout.read(&mut buf) {
                Ok(0) => break,
                Ok(1) => {
                    let b = buf[0];

                    let (value_opt, suppress) = observer.observe(b);

                    if !suppress {
                        stdout.write_all(&buf)?;
                        stdout.flush()?;
                    }

                    if let Some(v) = value_opt {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Ok(WrapperResult::Sentinel(v));
                    }
                }
                Ok(_) => unreachable!(),
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(e.into());
                }
            }
        }

        Ok(WrapperResult::Exit(child.wait()?))
    }
}
