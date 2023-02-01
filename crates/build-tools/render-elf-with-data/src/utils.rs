pub fn align_up(x: usize, n: usize) -> usize {
    match x {
        0 => 0,
        _ => ((x - 1) & !(n - 1)) + n,
    }
}
