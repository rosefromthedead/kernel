use byteorder::{ByteOrder, BE};
use cortex_a::interfaces::{Writeable, ReadWriteable};

use crate::println;
use crate::vm::{PhysicalAddress, VirtualAddress, Table};
use vm::table::{IntermediateLevel, IntermediateTable, Level0, Level1, Level2};

pub mod context;
pub mod interrupt;
pub mod memory;
pub mod vm;

pub const FRAME_SIZE: usize = 4096;

#[link_section = ".early_init"]
#[no_mangle]
#[naked]
unsafe extern "C" fn early_init() {
    asm!("
        adrp x4, EARLY_STACK
        add x4, x4, #0x1000
        mov sp, x4
        b {0}
    ", sym init, options(noreturn));
}

unsafe fn init() {
    // hope and pray that function prologue doesn't touch x0, because naked functions with stuff
    // actually in the body aren't good apparently
    let dtb_phys: usize;
    asm!("", out("x0") dtb_phys);
    let dtb_phys = PhysicalAddress(dtb_phys);

    // Initiailise system registers
    {
        use cortex_a::registers::*;
        // Don't trap on FP/SIMD register access
        CPACR_EL1.write(CPACR_EL1::TTA::None + CPACR_EL1::FPEN::None + CPACR_EL1::ZEN::None);
    }

    // Initialise MMU + translation tables
    {
        use cortex_a::registers::*;
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

        kernel_table.insert_raw(PhysicalAddress(kernel_remap_l1_addr), 0).unwrap();
        kernel_table.insert_raw(PhysicalAddress(direct_map_addr), 511).unwrap();
        kernel_remap_l1.insert_raw(PhysicalAddress(kernel_remap_l2_addr), 0).unwrap();
        kernel_remap_l2.insert_block(vm::KERNEL_LOAD_PHYS, 0).unwrap();
        for i in 0..511 {
            let phys = PhysicalAddress(i * Level1::BLOCK_SIZE as usize);
            direct_map.insert_block(phys, i).unwrap();
        }
        kernel_identity_l0.insert_raw(PhysicalAddress(kernel_identity_l1_addr), 0).unwrap();
        for i in 0..511 {
            let phys = PhysicalAddress(i * Level1::BLOCK_SIZE as usize);
            kernel_identity_l1.insert_block(phys, i).unwrap();
        }

        let mut tcr: u64;
        asm!("
            msr TTBR0_EL1, {0}
            msr TTBR1_EL1, {1}
            mrs {2}, TCR_EL1
        ", in(reg) kernel_identity_l0_addr, in(reg) kernel_table_addr, lateout(reg) tcr);
        tcr &= !(1 << 23);
        asm!("msr TCR_EL1, {0}", in(reg) tcr);

        SCTLR_EL1.modify(SCTLR_EL1::M::Enable);

        let par: u64;
        let addr: u64 = 0xFFFF_0000_0000_0000;
        asm!("
            at s1e1r, {0}
            mrs {1}, PAR_EL1
        ", in(reg) addr, lateout(reg) par);

        let difference = vm::KERNEL_OFFSET - vm::KERNEL_LOAD_PHYS.0;
        asm!("
            bl 1f
            1:
            add x30, x30, #(2f - 1b)
            add x30, x30, {0}
            br x30
            2:
        ", in(reg) difference);
        // now that we're in the right place in memory, we can use cortex_a funcs
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
    }

    println!("Hello, universe!");
    memory::init_heap();

    {
        let frames_ptr = &memory::SPARE_FRAMES as *const _;
        let par: usize;
        asm!("
            at s1e1r, {0}
            mrs {1}, PAR_EL1
        ", in(reg) frames_ptr, lateout(reg) par);
        let frames_phys = PhysicalAddress(par & 0x0000_FFFF_FFFF_F000);
        crate::memory::FRAME_ALLOCATOR.lock().insert_hole(frames_phys, 4096 * 8);

        vm::KERNEL_TABLE.map_to(VirtualAddress(0xFFFF_1000_0000_0000), dtb_phys, 4096).unwrap();
        let dt_header_bytes = core::slice::from_raw_parts(0xFFFF_1000_0000_0000 as *const _, 40);
        let dt_header = fdt::DeviceTreeHeader::new(&dt_header_bytes).unwrap();
        let dtb_size = dt_header.total_size as usize;
        vm::KERNEL_TABLE.unmap(VirtualAddress(0xFFFF_1000_0000_0000), 4096);
        vm::KERNEL_TABLE.map_to(VirtualAddress(0xFFFF_1000_0000_0000), dtb_phys, dtb_size).unwrap();

        let dtb = core::slice::from_raw_parts(0xFFFF_1000_0000_0000u64 as _, dtb_size);
        let dt = fdt::DeviceTree::new(&dtb).unwrap();

        let root = dt.nodes().next().unwrap();
        let address_cells = BE::read_u32(root.properties().find(|prop| prop.name == "#address-cells").unwrap().data);
        let size_cells = BE::read_u32(root.properties().find(|prop| prop.name == "#size-cells").unwrap().data);
        if address_cells != 2 || size_cells != 2 {
            println!("Problem");
            panic!();
        }

        let memory = dt.find_node("memory").unwrap();
        let reg = memory.properties().find(|prop| prop.name == "reg").unwrap();
        let start_addr = PhysicalAddress(BE::read_u64(&reg.data[0..8]) as usize);
        let size = BE::read_u64(&reg.data[8..16]) as usize;
        {
            let mut frame_allocator = crate::memory::FRAME_ALLOCATOR.lock();
            frame_allocator.insert_hole(start_addr, vm::KERNEL_LOAD_PHYS - start_addr);
            let kernel_end = vm::KERNEL_LOAD_PHYS + 1024 * 1024 * 2;
            frame_allocator.insert_hole(vm::KERNEL_LOAD_PHYS + 1024 * 1024 * 2, dtb_phys - kernel_end);
            let dtb_end = dtb_phys + dtb_size;
            frame_allocator.insert_hole(dtb_end, start_addr + size - dtb_end);
        }

        println!("Still alive");
    }

    interrupt::init_interrupts();

    let context = crate::context::Context::new();
    context.enter();
    context::very_good_context();
}
