pub(crate) trait SignedInteger {
    fn to_usize_or_zero(self) -> usize;
}

impl SignedInteger for i16 {
    fn to_usize_or_zero(self) -> usize {
        if self < 0 {
            0
        } else {
            self as usize
        }
    }
}

impl SignedInteger for i32 {
    fn to_usize_or_zero(self) -> usize {
        if self < 0 {
            0
        } else {
            self as usize
        }
    }
}

impl SignedInteger for i64 {
    fn to_usize_or_zero(self) -> usize {
        if self < 0 {
            0
        } else {
            self as usize
        }
    }
}
