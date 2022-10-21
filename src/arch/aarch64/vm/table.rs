use crate::vm::{PhysicalAddress, VirtualAddress, Table};
use alloc::boxed::Box;
use core::{fmt::{Debug, Formatter}, marker::PhantomData, mem::MaybeUninit};

use super::fmt::{debug_page_or_block, ForceUpperHex};

macro_rules! set_bit {
    ($value:expr, $bit:expr, $bool:expr) => {
        match $bool {
            true => *$value |= (1 << $bit),
            false => *$value &= !(1 << $bit),
        }
    }
}

fn phys_to_virt(phys: PhysicalAddress) -> VirtualAddress {
    let mask = 0xFFFF_FF80_0000_0000;
    let phys = phys.0;
    assert!(phys & mask == 0, "too much physical memory");
    VirtualAddress(phys | mask)
}

pub trait IntermediateLevel: Copy {
    type Next: Table + Debug + Default;
    const VIRT_SHIFT_AMT: u64;
    /// The size in bytes of one block at this table level, or the total number of bytes for which
    /// an entry at this block is responsible.
    const BLOCK_SIZE: u64;
    const BLOCK_ADDRESS_MASK: u64;
    const BLOCKS_SUPPORTED: bool;

    const IS_TOP_LEVEL: bool;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Level0;
impl IntermediateLevel for Level0 {
    type Next = IntermediateTable<Level1>;
    const VIRT_SHIFT_AMT: u64 = 39;

    const BLOCK_SIZE: u64 = 0x0000_0080_0000_0000;
    const BLOCK_ADDRESS_MASK: u64 = 0;
    const BLOCKS_SUPPORTED: bool = false;

    const IS_TOP_LEVEL: bool = true;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Level1;
impl IntermediateLevel for Level1 {
    type Next = IntermediateTable<Level2>;
    const VIRT_SHIFT_AMT: u64 = 30;
    // 1 GiB
    const BLOCK_SIZE: u64 = 0x4000_0000;
    const BLOCK_ADDRESS_MASK: u64 = 0x0000_FFFF_C000_0000;
    const BLOCKS_SUPPORTED: bool = true;

    const IS_TOP_LEVEL: bool = false;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Level2;
impl IntermediateLevel for Level2 {
    type Next = Level3Table;
    const VIRT_SHIFT_AMT: u64 = 21;
    // 2 MiB
    const BLOCK_SIZE: u64 = 0x0020_0000;
    const BLOCK_ADDRESS_MASK: u64 = 0x0000_FFFF_FFE0_0000;
    const BLOCKS_SUPPORTED: bool = true;

    const IS_TOP_LEVEL: bool = false;
}

#[repr(align(4096))]
pub struct IntermediateTable<L: IntermediateLevel> {
    entries: [IntermediateTableEntry<L>; 512],
    _tables: PhantomData<[Option<L::Next>; 512]>,
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
            let virt = Box::into_raw(next) as usize;
            let mut phys: usize;
            asm!("
                at s1e1r, {0}
                mrs {1}, PAR_EL1
            ", in(reg) virt, lateout(reg) phys);
            phys &= 0x0000_FFFF_FFFF_F000;
            self.force_insert(PhysicalAddress(phys), idx, true);
        }
        Ok(())
    }

    pub unsafe fn insert_raw(&mut self, next: PhysicalAddress, idx: usize) -> Result<(), ()> {
        if self.entries[idx].is_valid() {
            return Err(());
        }
        self.force_insert(next, idx, false);
        Ok(())
    }

    pub unsafe fn force_insert(&mut self, phys: PhysicalAddress, idx: usize, owned: bool) {
        let mut phys = phys.0;
        // assert_eq!(phys & 0xFFF, 0);
        phys &= 0x0000_FFFF_FFFF_F000;

        let mut entry = IntermediateTableEntry::new(PhysicalAddress(phys as usize));
        self.entries[idx] = entry;
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

    pub fn entry(&self, idx: usize) -> &IntermediateTableEntry<L> {
        &self.entries[idx]
    }

    pub unsafe fn entry_mut(&mut self, idx: usize) -> &mut IntermediateTableEntry<L> {
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

impl Debug for IntermediateTable<Level0> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for (i, entry) in self.entries.iter().enumerate().filter(|(_i, e)| e.is_valid()) {
            f.write_fmt(format_args!("{} {:#018x}:\n", i, entry.value))?;
            if let Some(next) = entry.get_next_table() {
                f.write_fmt(format_args!("{:?}", next))?;
            } else {
                panic!("level 0 entry must be invalid or table");
            }
        }
        Ok(())
    }
}

impl Debug for IntermediateTable<Level1> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for (i, entry) in self.entries.iter().enumerate().filter(|(_i, e)| e.is_valid()) {
            f.write_fmt(format_args!("  {} {:#018x}:\n", i, entry.value))?;
            if let Some(next) = entry.get_next_table() {
                f.write_fmt(format_args!("{:?}", next))?;
            } else {
                debug_page_or_block(entry, f)?;
            }
        }
        Ok(())
    }
}

impl Debug for IntermediateTable<Level2> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for (i, entry) in self.entries.iter().enumerate().filter(|(_i, e)| e.is_valid()) {
            f.write_fmt(format_args!("    {} {:#018x}:\n", i, entry.value))?;
            if let Some(next) = entry.get_next_table() {
                f.write_fmt(format_args!("{:?}", next))?;
            } else {
                debug_page_or_block(entry, f)?;
            }
        }
        Ok(())
    }
}

impl<L: IntermediateLevel> Table for IntermediateTable<L> {
    fn map_to(
        &mut self,
        virt: VirtualAddress,
        phys: PhysicalAddress,
        size: usize,
    ) -> Result<(), ()> {
        let starting_idx = ((virt.0 as u64 >> L::VIRT_SHIFT_AMT) & 0x1FF) as usize;
        let block_size = L::BLOCK_SIZE as usize;
        let mut entries_needed = (size + block_size - 1) / block_size;
        if starting_idx + entries_needed > 512 {
            entries_needed = 512 - starting_idx;
        }
        for i in 0..entries_needed {
            let idx = i + starting_idx;
            let entry = &mut self.entries[idx];
            let new_virt = VirtualAddress(virt.0 + i * L::BLOCK_SIZE as usize);
            let new_phys = PhysicalAddress(phys.0 + i * L::BLOCK_SIZE as usize);
            let new_size = size - i * L::BLOCK_SIZE as usize;
            match entry.get_next_table_mut() {
                Some(next_table) => {
                    return next_table.map_to(new_virt, new_phys, new_size);
                },
                None => {
                    let frame_phys = crate::memory::FRAME_ALLOCATOR.lock().alloc();
                    unsafe { self.insert_raw(frame_phys, idx)?; }
                    let next_table_uninit = unsafe { &mut *(phys_to_virt(frame_phys).0 as *mut MaybeUninit<L::Next>) };
                    let next_table: &mut L::Next = Table::clear(next_table_uninit);
                    next_table.map_to(new_virt, new_phys, new_size)?;
                },
            }
        }
        if L::IS_TOP_LEVEL {
            unsafe { asm!("
                dsb ishst
                isb
            "); }
        }
        Ok(())
    }

    fn unmap(&mut self, virt: VirtualAddress, size: usize) {
        let starting_idx = ((virt.0 as u64 >> L::VIRT_SHIFT_AMT) & 0x1FF) as usize;
        let block_size = L::BLOCK_SIZE as usize;
        let mut entries_needed = (size + block_size - 1) / block_size;
        if starting_idx + entries_needed > 512 {
            entries_needed = 512 - starting_idx;
        }
        for i in 0..entries_needed {
            let idx = i + starting_idx;
            let entry = &mut self.entries[idx];
            let new_virt = VirtualAddress(virt.0 + i * L::BLOCK_SIZE as usize);
            let new_size = size - i * L::BLOCK_SIZE as usize;
            if let Some(next_table) = entry.get_next_table_mut() {
                next_table.unmap(new_virt, new_size);
            }
        }
    }

    fn clear<'a>(this: &'a mut MaybeUninit<Self>) -> &'a mut Self {
        unsafe {
            // why even bother writing rust at this point
            let this_inner = core::mem::transmute::<
                _,
                &mut [MaybeUninit<u64>; 512],
            >(&mut *this);
            for entry in this_inner.iter_mut() {
                entry.write(0);
            }
            this.assume_init_mut()
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
            return f.debug_tuple("Table")
                .field(&ForceUpperHex(self.value))
                .field(unsafe { &*(phys_to_virt(table).0 as *const L::Next) })
                .finish()
        }
        if let Some(block) = self.block_address() {
            return f.debug_tuple("Block")
                .field(&ForceUpperHex(self.value))
                .field(&block)
                .finish()
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
        let mut value = phys | 0b11 | (1 << 2) | (1 << 10) | (1 << 6);
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

    pub(in crate::arch::aarch64) fn get_next_table(&self) -> Option<&L::Next> {
        match self.table_address() {
            Some(phys) => unsafe { Some(&*(phys_to_virt(phys).0 as *const _)) },
            None => None,
        }
    }

    pub(in crate::arch::aarch64) fn get_next_table_mut(&mut self) -> Option<&mut L::Next> {
        match self.table_address() {
            Some(phys) => unsafe { Some(&mut *(phys_to_virt(phys).0 as *mut _)) },
            None => None,
        }
    }
}

/// Functions for interacting with table attributes
impl<L: IntermediateLevel> IntermediateTableEntry<L> {
    pub fn get_ns(&self) -> bool {
        self.value & (1 << 63) != 0
    }

    pub fn set_ns(&mut self, value: bool) {
        set_bit!(&mut self.value, 63, value);
    }

    pub fn get_read_only(&self) -> bool {
        self.value & (1 << 62) != 0
    }

    pub fn set_read_only(&mut self, value: bool) {
        set_bit!(&mut self.value, 62, value);
    }

    pub fn get_el0_inaccessible(&self) -> bool {
        self.value & (1 << 61) != 0
    }

    pub fn set_el0_inaccessible(&mut self, value: bool) {
        set_bit!(&mut self.value, 61, value);
    }

    pub fn get_xn(&self) -> bool {
        self.value & (1 << 60) != 0
    }

    pub fn set_xn(&mut self, value: bool) {
        set_bit!(&mut self.value, 60, value);
    }

    pub fn get_pxn(&self) -> bool {
        self.value & (1 << 59) != 0
    }

    pub fn set_pxn(&mut self, value: bool) {
        set_bit!(&mut self.value, 59, value);
    }
}

/// Functions for interacting with block attributes
pub trait PageOrBlockDesc {
    fn value(&self) -> &u64;
    fn value_mut(&mut self) -> &mut u64;

    // Upper
    fn get_xn(&self) -> bool {
        self.value() & (1 << 54) != 0
    }

    fn set_xn(&mut self, value: bool) {
        set_bit!(self.value_mut(), 54, value);
    }

    fn get_pxn(&self) -> bool {
        self.value() & (1 << 53) != 0
    }

    fn set_pxn(&mut self, value: bool) {
        set_bit!(self.value_mut(), 53, value);
    }

    fn get_contiguous(&self) -> bool {
        self.value() & (1 << 52) != 0
    }

    fn set_contiguous(&mut self, value: bool) {
        set_bit!(self.value_mut(), 52, value);
    }

    fn get_dirty(&self) -> bool {
        self.value() & (1 << 51) != 0
    }

    fn set_dirty(&mut self, value: bool) {
        set_bit!(self.value_mut(), 51, value);
    }

    fn get_guarded(&self) -> bool {
        self.value() & (1 << 50) != 0
    }

    fn set_guarded(&mut self, value: bool) {
        set_bit!(self.value_mut(), 50, value);
    }

    // Custom
    fn get_owned(&self) -> bool {
        self.value() & (1 << 55) != 0
    }

    fn set_owned(&mut self, value: bool) {
        set_bit!(self.value_mut(), 55, value);
    }

    // Output address
    fn get_address(&self) -> u64 {
        // bits [47:12]
        self.value() & 0x0000_FFFF_FFFF_F000
    }

    // Lower
    fn get_nt(&self) -> bool {
        self.value() & (1 << 16) != 0
    }

    fn set_nt(&mut self, value: bool) {
        set_bit!(self.value_mut(), 16, value);
    }

    fn get_not_global(&self) -> bool {
        self.value() & (1 << 11) != 0
    }

    fn set_not_global(&mut self, value: bool) {
        set_bit!(self.value_mut(), 11, value);
    }

    fn get_access(&self) -> bool {
        self.value() & (1 << 10) != 0
    }

    fn set_access(&mut self, value: bool) {
        set_bit!(self.value_mut(), 10, value);
    }

    //TODO: shareable bits

    fn get_read_only(&self) -> bool {
        self.value() & (1 << 7) != 0
    }

    fn set_read_only(&mut self, value: bool) {
        set_bit!(self.value_mut(), 7, value);
    }

    fn get_el0_accessible(&self) -> bool {
        self.value() & (1 << 6) != 0
    }

    fn set_el0_accessible(&mut self, value: bool) {
        set_bit!(self.value_mut(), 6, value);
    }

    fn get_non_secure(&self) -> bool {
        self.value() & (1 << 5) != 0
    }

    fn set_non_secure(&mut self, value: bool) {
        set_bit!(self.value_mut(), 5, value);
    }

    //TODO: attrindex
}

impl PageOrBlockDesc for IntermediateTableEntry<Level1> {
    fn value(&self) -> &u64 {
        &self.value
    }

    fn value_mut(&mut self) -> &mut u64 {
        &mut self.value
    }
}

impl PageOrBlockDesc for IntermediateTableEntry<Level2> {
    fn value(&self) -> &u64 {
        &self.value
    }

    fn value_mut(&mut self) -> &mut u64 {
        &mut self.value
    }
}

#[repr(align(4096))]
pub struct Level3Table {
    entries: [Level3TableEntry; 512],
}

impl Level3Table {
    fn entry(&self, idx: usize) -> &Level3TableEntry {
        &self.entries[idx]
    }

    fn entry_mut(&mut self, idx: usize) -> &mut Level3TableEntry {
        &mut self.entries[idx]
    }

    fn get_frame_addr(&self, idx: usize) -> u64 {
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

impl Debug for Level3Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for (i, entry) in self.entries.iter().enumerate().filter(|(_i, e)| e.is_valid()) {
            f.write_fmt(format_args!("      {} {:#018x}: {:?}\n", i, entry.value, entry))?;
        }
        Ok(())
    }
}

impl Table for Level3Table {
    fn map_to(&mut self, virt: VirtualAddress, phys: PhysicalAddress, size: usize) -> Result<(), ()> {
        let start_idx = virt.0 >> 12 & 0x1FF;
        let entries_needed = (size + 4095) / 4096;
        for i in start_idx..start_idx + entries_needed {
            let entry = &mut self.entries[i];
            let new_phys = PhysicalAddress(phys.0 + i * 4096);
            *entry = Level3TableEntry::new(new_phys);
        }
        Ok(())
    }

    fn unmap(&mut self, virt: VirtualAddress, size: usize) {
        let starting_idx = virt.0 >> 12 & 0x1FF;
        let mut entries_to_remove = (size + 4095) / 4096;
        if entries_to_remove + starting_idx > 512 {
            entries_to_remove = 512 - starting_idx;
        }
        for i in starting_idx..starting_idx + entries_to_remove {
            self.entries[i] = Level3TableEntry::new_invalid();
        }
    }

    fn clear<'a>(this: &'a mut MaybeUninit<Self>) -> &'a mut Self {
        unsafe {
            // why even bother writing rust at this point
            let this_inner = core::mem::transmute::<
                _,
                &mut [MaybeUninit<u64>; 512],
            >(&mut *this);
            for entry in this_inner.iter_mut() {
                entry.write(0);
            }
            this.assume_init_mut()
        }
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
struct Level3TableEntry {
    value: u64,
}

impl Debug for Level3TableEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        debug_page_or_block(self, f)
    }
}

impl Level3TableEntry {
    const fn new(phys: PhysicalAddress) -> Self {
        let phys = phys.0 as u64 & 0x0000_FFFF_FFFF_F000;
        let mut value = phys | 0b11 | (1 << 2) | (1 << 10) | (1 << 6);
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

impl PageOrBlockDesc for Level3TableEntry {
    fn value(&self) -> &u64 {
        &self.value
    }

    fn value_mut(&mut self) -> &mut u64 {
        &mut self.value
    }
}

