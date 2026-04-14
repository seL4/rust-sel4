use alloc::boxed::Box;
use core::error::Error;

use sel4_microkit::{Channel, ChannelSet, Child, DeferredAction, Handler, MessageInfo, Never};

pub type UpcastedHandler = Box<dyn Handler<Error = Box<dyn Error>> + 'static>;

pub fn upcast_handler<E: Error + 'static>(
    handler: impl Handler<Error = E> + 'static,
) -> UpcastedHandler {
    Box::new(DynErrorHandlerWrapper(handler))
}

#[macro_export]
macro_rules! match_handler {
    {
        $(#[$attr:meta])*
        $fn_vis:vis fn $fn_ident:ident {
            $($pd_name:literal => $pd_init:expr,)*
        }
    } => {
        $(#[$attr])*
        $fn_vis fn $fn_ident() -> $crate::UpcastedHandler {
            match $crate::_with_alloc_private::pd_name().unwrap() {
                $($pd_name => $crate::upcast_handler($pd_init),)*
                _ => unreachable!(),
            }
        }
    };
}

#[doc(hidden)]
pub mod _with_alloc_private {
    pub use sel4_microkit::pd_name;
}

struct DynErrorHandlerWrapper<T>(T);

impl<T: Handler<Error: Error + 'static>> Handler for DynErrorHandlerWrapper<T> {
    type Error = Box<dyn Error>;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        self.0.notified(channels).map_err(Into::into)
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        self.0.protected(channel, msg_info).map_err(Into::into)
    }

    fn fault(
        &mut self,
        child: Child,
        msg_info: MessageInfo,
    ) -> Result<Option<MessageInfo>, Self::Error> {
        self.0.fault(child, msg_info).map_err(Into::into)
    }

    fn take_deferred_action(&mut self) -> Option<DeferredAction> {
        self.0.take_deferred_action()
    }

    #[doc(hidden)]
    fn run(&mut self) -> Result<Never, Self::Error> {
        self.0.run().map_err(Into::into)
    }
}
