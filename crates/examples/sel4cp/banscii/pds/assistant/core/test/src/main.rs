use std::io::{stdout, Write};

fn main() {
    env_logger::init();

    let subject = "Hello";

    banscii_assistant_core::draft(subject, |bytes| stdout().lock().write_all(bytes).unwrap());
}
