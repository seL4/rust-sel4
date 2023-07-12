use sel4_render_elf_with_data::{Input, SymbolicInjection, SymbolicValue};

pub fn render_elf(orig_elf: &[u8], data: &[u8], heap_size: usize) -> Vec<u8> {
    let align_modulus = 4096;
    let align_residue = (align_modulus - data.len() % align_modulus) % align_modulus;
    let memsz = data.len() + heap_size;
    let mut input = Input::default();
    input.symbolic_injections.push(SymbolicInjection {
        align_modulus,
        align_residue,
        content: data,
        memsz,
        patches: vec![
            (
                "sel4_capdl_initializer_serialized_spec_start".to_owned(),
                SymbolicValue { addend: 0 },
            ),
            (
                "sel4_capdl_initializer_heap_start".to_owned(),
                SymbolicValue {
                    addend: data.len().try_into().unwrap(),
                },
            ),
        ],
    });
    input
        .image_start_patches
        .push("sel4_capdl_initializer_image_start".to_owned());
    input
        .image_end_patches
        .push("sel4_capdl_initializer_image_end".to_owned());
    input.concrete_patches.push((
        "sel4_capdl_initializer_serialized_spec_size".to_owned(),
        data.len().try_into().unwrap(),
    ));
    input.concrete_patches.push((
        "sel4_capdl_initializer_heap_size".to_owned(),
        heap_size.try_into().unwrap(),
    ));
    input.render_with_data(orig_elf).unwrap()
}
