use sel4_backtrace::{BacktraceSendWithToken, BacktraceSendWithoutToken};
use sel4_runtime_building_blocks_abort::{debug_print, debug_println};

#[cfg(feature = "alloc")]
use sel4_backtrace::Backtrace;

struct SimpleBacktraceSend;

impl BacktraceSendWithoutToken for SimpleBacktraceSend {
    type Image = Option<&'static str>;
    type TxError = !;

    fn image(&self) -> Self::Image {
        None
    }

    fn send_byte(&self, byte: u8) -> Result<(), Self::TxError> {
        debug_print!("{:02x}", byte);
        Ok(())
    }
}

pub fn collect_and_send() {
    debug_println!("collecting and sending stack backtrace");
    debug_print!("    ");
    let r = SimpleBacktraceSend.collect_and_send().into_ok();
    debug_println!();
    debug_println!();
    if r.is_err() {
        debug_println!("error encountered while collecting and sending stack backtrace");
    }
}

#[cfg(feature = "alloc")]
pub fn collect() -> Backtrace<Option<&'static str>> {
    debug_println!("collecting stack backtrace");
    SimpleBacktraceSend.collect()
}

#[cfg(feature = "alloc")]
pub fn send(bt: &Backtrace<Option<&'static str>>) {
    debug_println!("sending stack backtrace");
    debug_print!("    ");
    let r = SimpleBacktraceSend.send(bt).into_ok();
    debug_println!();
    debug_println!();
    if r.is_err() {
        debug_println!("error encountered while sending stack backtrace");
    }
}
