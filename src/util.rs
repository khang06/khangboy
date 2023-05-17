// Rust doesn't have this by default :/
// https://github.com/rust-lang/rust/issues/82378
pub trait BitIndex {
    fn test(self, bit: u8) -> bool;
    fn set(self, bit: u8, val: bool) -> Self;
}

impl BitIndex for u8 {
    fn test(self, bit: u8) -> bool {
        self & (1 << bit) != 0
    }

    fn set(self, bit: u8, val: bool) -> Self {
        (self & !(1 << bit)) | ((val as u8) << bit)
    }
}

impl BitIndex for u16 {
    fn test(self, bit: u8) -> bool {
        self & (1 << bit) != 0
    }

    fn set(self, bit: u8, val: bool) -> Self {
        (self & !(1 << bit)) | ((val as u16) << bit)
    }
}
