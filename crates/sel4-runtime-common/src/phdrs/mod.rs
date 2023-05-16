mod elf;

#[cfg(feature = "embedded-phdrs")]
mod embedded;

#[cfg(feature = "embedded-phdrs")]
use embedded::locate_phdrs;

#[cfg(all(feature = "tls", target_thread_local))]
mod tls;

#[cfg(all(feature = "tls", target_thread_local))]
pub use tls::locate_tls_image;

#[cfg(feature = "unwinding")]
mod unwinding;

#[cfg(feature = "unwinding")]
pub use self::unwinding::set_eh_frame_finder;
