use num::{NumCast, One, PrimInt, Zero};
use object::{read::elf::FileHeader, Endianness};

use sel4_render_elf_with_data::{FileHeaderExt, Input, SymbolicInjection, SymbolicValue};

pub fn render_elf<T>(orig_elf: &[u8], serialized_payload: &[u8]) -> Vec<u8>
where
    T: FileHeaderExt + FileHeader<Word: PrimInt, Sword: PrimInt, Endian = Endianness>,
{
    let align_modulus = T::Word::one();
    let align_residue = T::Word::one();
    let memsz = serialized_payload.len();
    let mut input = Input::<T>::default();
    input.symbolic_injections.push(SymbolicInjection {
        align_modulus,
        align_residue,
        content: serialized_payload,
        memsz: NumCast::from(memsz).unwrap(),
        patches: vec![(
            "loader_payload_start".to_owned(),
            SymbolicValue {
                addend: T::Sword::zero(),
            },
        )],
    });
    input
        .image_start_patches
        .push("loader_image_start".to_owned());
    input.image_end_patches.push("loader_image_end".to_owned());
    input.concrete_patches.push((
        "loader_payload_size".to_owned(),
        NumCast::from(serialized_payload.len()).unwrap(),
    ));
    input.render_with_data(orig_elf).unwrap()
}
