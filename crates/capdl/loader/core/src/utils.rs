pub(crate) const fn round_up(n: usize, b: usize) -> usize {
    n.next_multiple_of(b)
}

pub(crate) const fn round_down(n: usize, b: usize) -> usize {
    n - n % b
}
