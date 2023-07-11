use core::fmt;

use sel4_panicking_env::debug_println;

pub trait InfallibleTermination {
    fn report(self) -> ();
}

impl InfallibleTermination for () {
    fn report(self) -> () {
        self
    }
}

impl InfallibleTermination for ! {
    fn report(self) -> () {
        self
    }
}

pub trait Termination {
    type Error;

    fn report(self) -> Result<(), Self::Error>;

    fn show(self) -> ()
    where
        Self: Sized,
        Self::Error: fmt::Debug,
    {
        self.report()
            .unwrap_or_else(|err| debug_println!("terminated with error: {err:?}"))
    }
}

impl<T: InfallibleTermination> Termination for T {
    type Error = !;

    fn report(self) -> Result<(), Self::Error> {
        Ok(self.report())
    }
}

impl<T: InfallibleTermination, E> Termination for Result<T, E> {
    type Error = E;

    fn report(self) -> Result<(), Self::Error> {
        self.map(InfallibleTermination::report)
    }
}
