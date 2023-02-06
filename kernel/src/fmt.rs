use core::fmt::{Formatter, LowerHex};

pub struct ForceLowerHex<T: LowerHex>(pub T);

impl<T: LowerHex> core::fmt::Debug for ForceLowerHex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
