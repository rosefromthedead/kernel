use alloc::{boxed::Box, collections::BTreeMap};
use cortex_a::regs::*;

use crate::println;
use crate::{console::virt::virt_putchar, vm::{PhysicalAddress, VirtualAddress, Table}};
use table::{IntermediateLevel, IntermediateTable, Level0, Level1, Level2};

mod table;

const KERNEL_OFFSET: usize = 0xFFFF_0000_0000_0000;
const KERNEL_LOAD_PHYS: usize = 0x4020_0000;

#[no_mangle]
static mut KERNEL_TABLE: IntermediateTable<Level0> = IntermediateTable::new();
#[no_mangle]
static mut KERNEL_REMAP_L1: IntermediateTable<Level1> = IntermediateTable::new();
#[no_mangle]
static mut KERNEL_REMAP_L2: IntermediateTable<Level2> = IntermediateTable::new();
#[no_mangle]
static mut DIRECT_MAP: IntermediateTable<Level1> = IntermediateTable::new();
#[no_mangle]
static mut KERNEL_IDENTITY_L0: IntermediateTable<Level0> = IntermediateTable::new();
#[no_mangle]
static mut KERNEL_IDENTITY_L1: IntermediateTable<Level1> = IntermediateTable::new();

pub struct VirtMem {
    user_tables: BTreeMap<usize, Box<IntermediateTable<Level0>>>,
}

impl VirtMem {
    // pub unsafe fn take_from_efi(st: &SystemTable<Boot>) -> () {
    //     let stdout = st.stdout();

    //     // EFI hands us the CPU with MMU on, and identity mapping for the memory that is in use.
    //     let mut ttbr0_el1: u64;
    //     asm!("
    //         mrs {0}, TTBR0_EL1
    //     ", out(reg) ttbr0_el1);
    //     let user_table_ptr: *mut IntermediateTable<Level0> = ttbr0_el1 as usize as *mut _;
    //     //TODO: deallocate the user table, even though we didn't allocate it

    //     let size = st.boot_services().memory_map_size();
    //     let mut buffer = alloc::vec![0; size];
    //     let (mmap_key, mut mmap_iter) = st.boot_services().memory_map(&mut buffer).unwrap_success();
    //     let (kern_addr, kern_size_pages) = mmap_iter
    //         .find(|x| x.ty == MemoryType::LOADER_CODE)
    //         .map(|entry| (entry.phys_start, entry.page_count))
    //         .unwrap();
    //     writeln!(stdout, "kernel at: {:#018X}, {} pages", kern_addr, kern_size_pages);

    //     // these shouldn't be necessary but they're getting filled with garbage for some reason
    //     KERNEL_TABLE.clear();
    //     DIRECT_MAP.clear();
        
    //     for i in 0..512 {
    //         let phys = PhysicalAddress(i * Level1::BLOCK_SIZE as usize);
    //         DIRECT_MAP.insert_block(phys, i).unwrap();
    //     }
    //     writeln!(stdout, "{:?}", KERNEL_TABLE.insert_raw(PhysicalAddress(&mut DIRECT_MAP as *mut _ as _), 511));
    //     for i in 0..512 {
    //         writeln!(stdout, "{:?}", DIRECT_MAP.entry_mut(i));
    //     }
            
    //     let kernel_table_addr = &mut KERNEL_TABLE as *mut _ as usize;
    //     let mut tcr: u64;
    //     let mut test: u64 = 0xFFFF_0000_0001_0000;
    //     let par: u64;
    //     asm!("
    //         msr TTBR1_EL1, {0}
    //         mrs {1}, TCR_EL1
    //         at s1e1r, {2}
    //         mrs {3}, PAR_EL1
    //     ", in(reg) kernel_table_addr, out(reg) tcr, in(reg) test, lateout(reg) par);
    //     writeln!(stdout, "PAR_EL1: {:#018X}", par);

    //     tcr &= !(1 << 23);

    //     asm!("msr TCR_EL1, {0}", in(reg) tcr);

    //     let kern_page = kern_addr / 4096;
    //     for (idx, page) in (kern_page..kern_page + kern_size_pages).enumerate() {
    //         let phys = PhysicalAddress(page as usize * 4096);
    //         let virt = VirtualAddress(KERNEL_OFFSET + idx * 4096);
    //         KERNEL_TABLE.map_to(virt, phys);
    //     }

    //     let sp: u64;
    //     asm!("
    //         mov {0}, sp
    //     ", out(reg) sp);
    //     writeln!(stdout, "{:#018X}", sp);

    //     let difference = KERNEL_OFFSET - kern_addr as usize;
    //     asm!("
    //         bl 1f
    //         1:
    //         add x30, x30, #(2f - 1b)
    //         add x30, x30, {0}
    //         br x30
    //         2:
    //     ", in(reg) difference);

    //     writeln!(stdout, "Successfully remapped the kernel");
    //     loop {
    //         asm!("wfi");
    //     }
    // }

    pub unsafe fn init() {
        virt_putchar(b'a');
        let kernel_table_addr: usize;
        let kernel_remap_l1_addr: usize;
        let kernel_remap_l2_addr: usize;
        let direct_map_addr: usize;
        let kernel_identity_l0_addr: usize;
        let kernel_identity_l1_addr: usize;
        asm!("
            adrp {0}, KERNEL_TABLE
            adrp {1}, KERNEL_REMAP_L1
            adrp {2}, KERNEL_REMAP_L2
            adrp {3}, DIRECT_MAP
            adrp {4}, KERNEL_IDENTITY_L0
            adrp {5}, KERNEL_IDENTITY_L1
        ", out(reg) kernel_table_addr, out(reg) kernel_remap_l1_addr,
            out(reg) kernel_remap_l2_addr, out(reg) direct_map_addr,
            out(reg) kernel_identity_l0_addr, out(reg) kernel_identity_l1_addr
        );
        let kernel_table = &mut *(kernel_table_addr as *mut IntermediateTable<Level0>);
        let kernel_remap_l1 = &mut *(kernel_remap_l1_addr as *mut IntermediateTable<Level1>);
        let kernel_remap_l2 = &mut *(kernel_remap_l2_addr as *mut IntermediateTable<Level2>);
        let direct_map = &mut *(direct_map_addr as *mut IntermediateTable<Level1>);
        let kernel_identity_l0 = &mut *(kernel_identity_l0_addr as *mut IntermediateTable<Level0>);
        let kernel_identity_l1 = &mut *(kernel_identity_l1_addr as *mut IntermediateTable<Level1>);

        kernel_table.clear();
        kernel_remap_l1.clear();
        kernel_remap_l2.clear();
        direct_map.clear();
        kernel_identity_l0.clear();
        kernel_identity_l1.clear();
        virt_putchar(b'a');

        kernel_table.insert_raw(PhysicalAddress(kernel_remap_l1_addr), 0).unwrap();
        virt_putchar(b'b');
        kernel_table.insert_raw(PhysicalAddress(direct_map_addr), 511).unwrap();
        virt_putchar(b'b');
        kernel_remap_l1.insert_raw(PhysicalAddress(kernel_remap_l2_addr), 0).unwrap();
        virt_putchar(b'b');
        kernel_remap_l2.insert_block(PhysicalAddress(KERNEL_LOAD_PHYS), 0).unwrap();
        virt_putchar(b'b');
        for i in 0..511 {
            let phys = PhysicalAddress(i * Level1::BLOCK_SIZE as usize);
            direct_map.insert_block(phys, i).unwrap();
        }
        virt_putchar(b'b');
        kernel_identity_l0.insert_raw(PhysicalAddress(kernel_identity_l1_addr), 0).unwrap();
        for i in 0..511 {
            let phys = PhysicalAddress(i * Level1::BLOCK_SIZE as usize);
            kernel_identity_l1.insert_block(phys, i).unwrap();
        }
        virt_putchar(b'b');

        // let mut tcr: u64;
        // asm!("
        //     msr TTBR0_EL1, {0}
        //     msr TTBR1_EL1, {1}
        //     mrs {2}, TCR_EL1
        // ", in(reg) kernel_identity_l0_addr, in(reg) kernel_table_addr, lateout(reg) tcr);
        // tcr &= !(1 << 23);
        // asm!("msr TCR_EL1, {0}", in(reg) tcr);
        virt_putchar(b'a');

        TTBR0_EL1.set_baddr(kernel_identity_l0_addr as u64);
        TTBR1_EL1.set_baddr(kernel_table_addr as u64);
        TCR_EL1.write(TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::EPD1::EnableTTBR1Walks
            + TCR_EL1::IPS::Bits_48
            + TCR_EL1::IRGN0::NonCacheable
            + TCR_EL1::IRGN1::NonCacheable
            + TCR_EL1::ORGN0::NonCacheable
            + TCR_EL1::ORGN1::NonCacheable
            + TCR_EL1::SH0::None
            + TCR_EL1::SH1::None
            + TCR_EL1::T0SZ.val(16)
            + TCR_EL1::T1SZ.val(16)
            + TCR_EL1::TBI0::Used
            + TCR_EL1::TBI1::Used
            + TCR_EL1::TG0::KiB_4
            + TCR_EL1::TG1::KiB_4);
        virt_putchar(b'a');
        
        SCTLR_EL1.modify(SCTLR_EL1::M::Enable);
        virt_putchar(b'!');

        let par: u64;
        let addr: u64 = 0xFFFF_0000_0000_0000;
        asm!("
            at s1e1r, {0}
            mrs {1}, PAR_EL1
        ", in(reg) addr, lateout(reg) par);


        let difference = KERNEL_OFFSET - KERNEL_LOAD_PHYS as usize;
        asm!("
            bl 1f
            1:
            add x30, x30, #(2f - 1b)
            add x30, x30, {0}
            br x30
            2:
        ", in(reg) difference);
        virt_putchar(b'!');
    }
}
