/// See [Xorshift](https://en.wikipedia.org/wiki/Xorshift) for more information
#[inline(always)]
pub fn next_random(value: &mut u64) -> u64 {
    let mut x = *value;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *value = x;
    x
}

#[inline(always)]
pub fn next_random_bool(value: &mut u64) -> bool {
    next_random(value) & 1 == 1
}
