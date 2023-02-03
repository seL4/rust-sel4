use render_elf_with_data::{inject, BoundsSymbolArgs, SymbolicInjection, DEFAULT_ALIGN};

pub fn render_elf(orig_elf: &[u8], data: &[u8], start_symbol: &str, size_symbol: &str) -> Vec<u8> {
    let image_bounds_symbols = BoundsSymbolArgs {
        start: vec![],
        end: vec![],
        size: vec![],
    };
    let injection_bounds_symbols = BoundsSymbolArgs {
        start: vec![start_symbol.to_owned()],
        end: vec![],
        size: vec![size_symbol.to_owned()],
    };
    let injection = SymbolicInjection::new(DEFAULT_ALIGN, data.to_vec(), &injection_bounds_symbols);
    inject(orig_elf, vec![injection], &image_bounds_symbols).unwrap()
}
