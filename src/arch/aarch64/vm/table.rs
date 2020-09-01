use crate::{console::virt::virt_putchar, vm::{PhysicalAddress, VirtualAddress, Table}};
use alloc::boxed::Box;
use bitflags::bitflags;
use core::{fmt::{Debug, Formatter}, marker::PhantomData};

fn phys_to_virt(phys: PhysicalAddress) -> VirtualAddress {
    // let mask = 0xFFFF_FF80_0000_0000;
    // let phys = phys.0;
    // assert!(phys & mask == 0, "too much physical memory");
    // VirtualAddress(phys | mask)
    VirtualAddress(phys.0)
}

bitflags! {
    struct TableDescriptor: u64 {
        const NS_TABLE = 1 << 63;
        const AP_TABLE = 0b11 << 61;
        const XN_TABLE = 1 << 60;
        const PXN_TABLE = 1 << 59;

        // [58:52] ignored. [51:48] res0. [47:12] next-level table address. [11:2] ignored.
        // The other two bits tell us which format the descriptor is in. 0b11 is this format.
        const OWNED = 1 << 58;

        const ADDRESS_MASK = 0x0000_FFFF_FFFF_F000;

        const FLAGS_MASK = 0b11111 << 59;
        const NONE = 0;
    }
}

bitflags! {
    struct PageDescriptor: u64 {
        // 63 ignored. [62:59] PBHA. [58:55] ignored.
        const XN = 1 << 54;
        const PXN = 1 << 53;
        const CONTIGUOUS = 1 << 52;
        const DIRTY = 1 << 51;
        // The spec claims that 50 is both GUARDED and res0, so I assume it's the former.
        const GUARDED = 1 << 50;

        // [49:48] res0.

        const NOT_GLOBAL = 1 << 11;
        const ACCESS = 1 << 10;
        const SHAREABILITY = 0b11 << 8;
        const ACCESS_PERMISSIONS = 0b11 << 6;
        const NON_SECURE = 1 << 5;
        const ATTR_INDEX = 0b111 << 2;
        const FLAGS_MASK = 0b11111 << 50 & 0b00001111_11111100;
        const NONE = 0;
    }
}

pub trait IntermediateLevel: Copy {
    type Next: Table + Debug + Default;
    const VIRT_SHIFT_AMT: u64;
    /// The size in bytes of one block at this table level, or 0 if blocks are not allowed.
    const BLOCK_SIZE: u64;
    const BLOCK_ADDRESS_MASK: u64;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Level0;
impl IntermediateLevel for Level0 {
    type Next = IntermediateTable<Level1>;
    const VIRT_SHIFT_AMT: u64 = 39;
    // Blocks are not supported at Level 0
    const BLOCK_SIZE: u64 = 0;
    const BLOCK_ADDRESS_MASK: u64 = 0;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Level1;
impl IntermediateLevel for Level1 {
    type Next = IntermediateTable<Level2>;
    const VIRT_SHIFT_AMT: u64 = 30;
    // 1 GiB
    const BLOCK_SIZE: u64 = 0x4000_0000;
    const BLOCK_ADDRESS_MASK: u64 = 0x0000_FFFF_C000_0000;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Level2;
impl IntermediateLevel for Level2 {
    type Next = Level3Table;
    const VIRT_SHIFT_AMT: u64 = 21;
    // 2 MiB
    const BLOCK_SIZE: u64 = 0x0020_0000;
    const BLOCK_ADDRESS_MASK: u64 = 0x0000_FFFF_FFE0_0000;
}

#[repr(align(4096))]
#[derive(Debug)]
pub struct IntermediateTable<L: IntermediateLevel> {
    entries: [IntermediateTableEntry<L>; 512],
    // there will be between 0 and 512 of these, but I think this is good enough
    _tables: PhantomData<L::Next>,
}

impl<L: IntermediateLevel> IntermediateTable<L> {
    pub const fn new() -> Self {
        Self {
            entries: [IntermediateTableEntry::<L>::new_invalid(); 512],
            _tables: PhantomData,
        }
    }

    pub fn insert(&mut self, next: Box<L::Next>, idx: usize) -> Result<(), ()> {
        if self.entries[idx].is_valid() {
            return Err(());
        }
        unsafe {
            self.force_insert(PhysicalAddress(Box::into_raw(next) as _), idx, true);
        }
        Ok(())
    }

    pub unsafe fn insert_raw(&mut self, next: PhysicalAddress, idx: usize) -> Result<(), ()> {
        if self.entries[idx].is_valid() {
            return Err(());
        }
        virt_putchar(b'b');
        self.force_insert(next, idx, false);
        virt_putchar(b'b');
        Ok(())
    }

    pub unsafe fn force_insert(&mut self, phys: PhysicalAddress, idx: usize, owned: bool) {
        virt_putchar(b'b');
        let mut phys = phys.0;
        // assert_eq!(phys & 0xFFF, 0);
        phys &= 0x0000_FFFF_FFFF_F000;
        virt_putchar(b'b');

        self.entries[idx] =
            IntermediateTableEntry::new(PhysicalAddress(phys as usize)).with_owned(owned);
        virt_putchar(b'b');
    }

    pub fn insert_block(&mut self, block: PhysicalAddress, idx: usize) -> Result<(), ()> {
        if self.entries[idx].is_valid() {
            return Err(());
        }
        unsafe {
            self.force_insert_block(block, idx);
        }
        Ok(())
    }

    pub unsafe fn force_insert_block(&mut self, phys: PhysicalAddress, idx: usize) {
        self.entries[idx] = IntermediateTableEntry::new_block(phys);
    }

    pub fn get_next_level_mut(&mut self, idx: usize) -> Option<&mut L::Next> {
        let entry = &self.entries[idx];
        match entry.table_address() {
            Some(phys) => unsafe { Some(&mut *(phys_to_virt(phys).0 as *mut _)) },
            None => None,
        }
    }

    pub fn entry_mut(&mut self, idx: usize) -> &mut IntermediateTableEntry<L> {
        &mut self.entries[idx]
    }
}

impl<L: IntermediateLevel> Default for IntermediateTable<L> {
    fn default() -> Self {
        Self {
            entries: [IntermediateTableEntry::new_invalid(); 512],
            _tables: PhantomData,
        }
    }
}

impl<L: IntermediateLevel> Table for IntermediateTable<L> {
    fn map_to(&mut self, virt: VirtualAddress, phys: PhysicalAddress) -> Result<(), ()> {
        let idx = ((virt.0 as u64 >> L::VIRT_SHIFT_AMT) & 0x1FF) as usize;
        let next_table = self.get_next_level_mut(idx);
        match next_table {
            Some(next_table) => return next_table.map_to(virt, phys),
            None => {
                // desperately avoiding using stack space because we don't have much
                // and page tables are quite large
                // this is safe because the compiler assumes nothing about any of the table types
                let mut next_table: Box<L::Next> = unsafe { Box::new_uninit().assume_init() };
                next_table.clear();
                next_table.map_to(virt, phys)?;
                self.insert(next_table, idx)?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = IntermediateTableEntry::new_invalid();
        }
    }
}

#[derive(Copy, Clone)]
pub struct IntermediateTableEntry<L: IntermediateLevel> {
    value: u64,
    _marker: PhantomData<L>,
}

impl<L: IntermediateLevel> Debug for IntermediateTableEntry<L> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        if !self.is_valid() {
            return write!(f, "{:?}", Option::<()>::None);
        }
        if let Some(table) = self.table_address() {
            return f.debug_tuple("Table").field(unsafe { &*(phys_to_virt(table).0 as *const L::Next) }).finish()
        }
        if let Some(block) = self.block_address() {
            return f.debug_tuple("Block").field(&block).finish()
        }
        unreachable!()
    }
}

impl<L: IntermediateLevel> IntermediateTableEntry<L> {
    pub fn value(&self) -> u64 {
        self.value
    }

    fn new(phys: PhysicalAddress) -> Self {
        let phys = phys.0 as u64 & 0x0000_FFFF_FFFF_F000;
        let value = phys | 0b11;
        Self {
            value,
            _marker: PhantomData,
        }
    }

    fn new_block(phys: PhysicalAddress) -> Self {
        let phys = phys.0 as u64 & L::BLOCK_ADDRESS_MASK;
        let value = phys | 0b01 | (1 << 10);
        Self {
            value,
            _marker: PhantomData,
        }
    }

    const fn new_invalid() -> Self {
        Self {
            value: 0,
            _marker: PhantomData,
        }
    }

    const fn is_valid(&self) -> bool {
        self.value & 1 == 1
    }

    fn is_table(&self) -> bool {
        self.value & 0b11 == 0b11
    }

    fn is_block(&self) -> bool {
        self.value & 0b11 == 0b01
    }

    fn table_address(&self) -> Option<PhysicalAddress> {
        match self.is_table() {
            true => Some(PhysicalAddress((self.value & 0x0000_FFFF_FFFF_F000) as _)),
            false => None,
        }
    }

    fn block_address(&self) -> Option<PhysicalAddress> {
        match self.is_block() {
            true => Some(PhysicalAddress((self.value & L::BLOCK_ADDRESS_MASK) as _)),
            false => None,
        }
    }

    fn with_owned(mut self, owned: bool) -> Self {
        let flag = /* TableDescriptor::OWNED.bits() */ 0;
        match owned {
            true => self.value |= flag,
            false => self.value &= !flag,
        }
        self
    }
}

#[repr(align(4096))]
#[derive(Debug)]
pub struct Level3Table {
    entries: [Level3TableEntry; 512],
}

impl Level3Table {
    pub fn get_frame_addr(&self, idx: usize) -> u64 {
        self.entries[idx].address()
    }
}

impl Default for Level3Table {
    fn default() -> Self {
        Self {
            entries: [Level3TableEntry::new_invalid(); 512],
        }
    }
}

impl Table for Level3Table {
    fn map_to(&mut self, virt: VirtualAddress, phys: PhysicalAddress) -> Result<(), ()> {
        let idx = virt.0 >> 12 & 0x1FF as usize;
        if self.entries[idx].is_valid() {
            return Err(());
        }
        self.entries[idx] = Level3TableEntry::new(phys);
        Ok(())
    }

    fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = Level3TableEntry::new_invalid();
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Level3TableEntry {
    value: u64,
}

impl Level3TableEntry {
    const fn new(phys: PhysicalAddress) -> Self {
        let phys = phys.0 as u64 & 0x0000_FFFF_FFFF_F000;
        let value = phys | 0b11 | (1 << 10) | 4;
        Self { value }
    }

    const fn new_invalid() -> Self {
        Self { value: 0 }
    }

    fn is_valid(&self) -> bool {
        self.value & 0b11 == 0b11
    }

    fn address(&self) -> u64 {
        self.value & 0x0000_FFFF_FFFF_F000
    }
}
