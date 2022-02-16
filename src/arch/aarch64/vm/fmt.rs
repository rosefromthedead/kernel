use core::fmt::{Formatter, UpperHex};

use super::table::PageOrBlockDesc;

pub struct ForceUpperHex<T: UpperHex>(pub T);

impl<T: UpperHex> core::fmt::Debug for ForceUpperHex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

pub fn debug_page_or_block(v: &impl PageOrBlockDesc, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("PageOrBlockDesc")
        .field("xn", &v.get_xn())
        .field("pxn", &v.get_pxn())
        .field("contiguous", &v.get_contiguous())
        .field("dirty", &v.get_dirty())
        .field("guarded", &v.get_guarded())
        .field("address", &ForceUpperHex(v.get_address()))
        .field("not_global", &v.get_not_global())
        .field("access", &v.get_access())
        .field("read_only", &v.get_read_only())
        .field("el0_accessible", &v.get_el0_accessible())
        .field("non_secure", &v.get_non_secure())
        .finish()
}
