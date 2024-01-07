//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use banscii_assistant_core::Draft;

fn main() {
    env_logger::init();

    let subject = "Hello, World!";

    let draft = Draft::new(subject);

    for row in 0..draft.height {
        for col in 0..draft.width {
            let i = row * draft.width + col;
            let grey = draft.pixel_data[i];
            let color = colorize(grey);
            print!("{}", color as char);
        }
        println!();
    }
}

const PALETTE: &[u8] = b"@%#x+=:-. ";

fn colorize(grey: u8) -> u8 {
    PALETTE
        [usize::from(grey) / (usize::from(u8::MAX).next_multiple_of(PALETTE.len()) / PALETTE.len())]
}
