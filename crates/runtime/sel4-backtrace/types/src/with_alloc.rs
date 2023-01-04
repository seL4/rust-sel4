use alloc::vec;
use alloc::vec::Vec;

#[cfg(feature = "postcard")]
use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Backtrace<T> {
    pub preamble: Preamble<T>,
    pub entries: Vec<Entry>,
    pub postamble: Postamble,
}

#[cfg(feature = "postcard")]
impl<T: Serialize> Backtrace<T> {
    pub fn send_to_vec(&self) -> postcard::Result<Vec<u8>> {
        let mut acc = vec![];
        let mut send_byte = |b| Result::<(), !>::Ok(acc.push(b));
        self.preamble.send(&mut send_byte)?;
        for entry in &self.entries {
            entry.send(&mut send_byte)?;
        }
        self.postamble.send(&mut send_byte)?;
        Ok(acc)
    }
}

#[cfg(feature = "postcard")]
impl<T: for<'a> Deserialize<'a>> Backtrace<T> {
    pub fn recv_taking(bytes: &[u8]) -> postcard::Result<(Self, &[u8])> {
        let mut buf = bytes;
        let preamble = take_from_bytes(&mut buf)?;
        let mut entries = vec![];
        loop {
            let more = take_from_bytes(&mut buf)?;
            if more {
                let entry = take_from_bytes(&mut buf)?;
                entries.push(entry);
            } else {
                break;
            }
        }
        let postamble = take_from_bytes(&mut buf)?;
        let backtrace = Self {
            preamble,
            entries,
            postamble,
        };
        Ok((backtrace, buf))
    }

    pub fn recv_taking_all(bytes: &[u8]) -> postcard::Result<Self> {
        let (backtrace, rem) = Self::recv_taking(bytes)?;
        if rem.is_empty() {
            Ok(backtrace)
        } else {
            Err(postcard::Error::SerdeDeCustom)
        }
    }

    pub fn recv(bytes: &[u8]) -> postcard::Result<Self> {
        let (backtrace, _) = Self::recv_taking(bytes)?;
        Ok(backtrace)
    }
}

impl<T> Backtrace<T> {
    pub fn builder(image: T) -> Builder<T> {
        Builder {
            preamble: Preamble { image },
            entries: vec![],
        }
    }
}

#[cfg(feature = "postcard")]
fn take_from_bytes<T: for<'a> Deserialize<'a>>(buf: &mut &[u8]) -> postcard::Result<T> {
    let (v, rem) = postcard::take_from_bytes(*buf)?;
    *buf = rem;
    Ok(v)
}

pub struct Builder<T> {
    preamble: Preamble<T>,
    entries: Vec<Entry>,
}

impl<T> Builder<T> {
    pub fn append(&mut self, entry: Entry) {
        self.entries.push(entry)
    }

    pub fn finalize(self, error: Option<Error>) -> Backtrace<T> {
        Backtrace {
            preamble: self.preamble,
            entries: self.entries,
            postamble: Postamble { error },
        }
    }
}

#[cfg(feature = "postcard")]
impl<T> Backtrace<T> {
    pub fn display_hex(&self) -> DisplayHex<T> {
        DisplayHex(self)
    }
}

#[cfg(feature = "postcard")]
pub struct DisplayHex<'a, T>(&'a Backtrace<T>);

#[cfg(feature = "postcard")]
impl<'a, T: Serialize> fmt::Display for DisplayHex<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for b in self.0.send_to_vec().map_err(|_| fmt::Error)? {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use alloc::borrow::ToOwned;
    use alloc::string::String;

    use super::*;

    #[test]
    fn test() {
        let bt = Backtrace::<String> {
            preamble: Preamble {
                image_path: "foo".to_owned(),
            },
            entries: vec![
                Entry {
                    stack_frame: StackFrame { ip: 123 },
                },
                Entry {
                    stack_frame: StackFrame { ip: 456 },
                },
            ],
            postamble: Postamble { error: None },
        };
        let bytes = bt.send_to_vec().unwrap();
        std::eprintln!("x {:?}", bytes);
        let reflected = Backtrace::<String>::recv_taking_all(&bytes).unwrap();
        assert_eq!(bt, reflected);
    }
}
