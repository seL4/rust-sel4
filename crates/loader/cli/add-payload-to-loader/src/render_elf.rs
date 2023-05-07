use render_elf_with_data::{Input, SymbolicInjection, SymbolicValue};

pub fn render_elf(orig_elf: &[u8], serialized_payload: &[u8]) -> Vec<u8> {
    let align_modulus = 1;
    let align_residue = 1;
    let memsz = serialized_payload.len();
    let mut input = Input::default();
    input.symbolic_injections.push(SymbolicInjection {
        align_modulus,
        align_residue,
        content: serialized_payload,
        memsz,
        patches: vec![(
            "loader_payload_start".to_owned(),
            SymbolicValue { addend: 0 },
        )],
    });
    input
        .image_start_patches
        .push("loader_image_start".to_owned());
    input.image_end_patches.push("loader_image_end".to_owned());
    input.concrete_patches.push((
        "loader_payload_size".to_owned(),
        serialized_payload.len().try_into().unwrap(),
    ));
    input.render_with_data(orig_elf).unwrap()
}
