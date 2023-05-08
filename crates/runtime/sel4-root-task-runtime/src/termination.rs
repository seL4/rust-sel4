pub trait Termination {
    type Error;

    fn report(self) -> Self::Error;
}

impl Termination for ! {
    type Error = !;

    fn report(self) -> Self::Error {
        self
    }
}

impl<E> Termination for Result<!, E> {
    type Error = E;

    fn report(self) -> Self::Error {
        self.into_err()
    }
}
