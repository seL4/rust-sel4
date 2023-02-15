use banscii_assistant_core::Draft;

fn main() {
    env_logger::init();

    let subject = "Hello";

    let draft = Draft::new(subject);

    let palette = b"@%#x+=:-. ";
    for row in 0..draft.height {
        for col in 0..draft.width {
            let i = row * draft.width + col;
            let v = draft.pixel_data[i];
            let c = palette[usize::from(v / 26)];
            print!("{}", c as char);
        }
        println!();
    }
}
