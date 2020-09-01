#[derive(Copy, Clone, Debug)]
pub struct PhysicalAddress(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct VirtualAddress(pub usize);

pub trait Table {
    /// Returns Err(()) and doesn't map anything if this virtual address is already mapped
    fn map_to(&mut self, virt: VirtualAddress, phys: PhysicalAddress) -> Result<(), ()>;
    fn clear(&mut self);
}
