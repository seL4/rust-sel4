pub use embedded_fat as fat;

pub struct DummyTimeSource(());

impl DummyTimeSource {
    pub fn new() -> Self {
        Self(())
    }
}

impl fat::TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> fat::Timestamp {
        unimplemented!()
    }
}
