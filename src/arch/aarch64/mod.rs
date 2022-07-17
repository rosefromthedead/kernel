use alloc::string::String;
use byteorder::{ByteOrder, BE};
use cortex_a::interfaces::{Writeable, ReadWriteable};
use tracing::info;

use crate::arch::vm::{KERNEL_LOAD_PHYS, KERNEL_TABLE};
use crate::memory::KERNEL_HEAP_ALLOCATOR;
use crate::println;
use crate::vm::{PhysicalAddress, VirtualAddress, Table};
use vm::table::{IntermediateLevel, IntermediateTable, Level0, Level1, Level2};

pub mod context;
pub mod interrupt;
pub mod memory;
pub mod vm;
mod regs;
pub mod platform;

pub const FRAME_SIZE: usize = 4096;

pub struct Arch {
    device_tree: fdt::DeviceTree<'static>,
    pub initrd: &'static [u8],
}

#[link_section = ".early_init"]
#[no_mangle]
#[naked]
unsafe extern "C" fn early_init() {
    asm!("
        adrp x4, EARLY_STACK
        add x4, x4, #0x2000
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
        kernel_remap_l2.insert_block(KERNEL_LOAD_PHYS, 0).unwrap();
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

        let difference = vm::KERNEL_OFFSET - vm::KERNEL_LOAD_PHYS.0;
        asm!("
            add sp, sp, {0}

            bl 1f
            1:
            add x30, x30, #(2f - 1b)
            add x30, x30, {0}
            br x30
            2:
        ", in(reg) difference, out("x30") _);
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

        MAIR_EL1.write(MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc);
    }

    memory::init_early_heap(&mut KERNEL_TABLE);

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

    let (memory, memory_cells) = dt.find_node("/memory").unwrap();
    assert_eq!(memory_cells.address, 2);
    assert_eq!(memory_cells.size, 2);
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

    KERNEL_TABLE.map_to(VirtualAddress(0xFFFF_FF00_0000_0000), PhysicalAddress(0x0000_0000_0900_0000), 4096).unwrap();
    tracing::subscriber::set_global_default(crate::tracing::PutcharSubscriber::new()).unwrap();
    let span = tracing::info_span!("kernel entry point");
    let _guard = span.enter();
    info!("Hello, universe!");
    interrupt::init_interrupts();

    let (chosen, _chosen_cells) = dt.find_node("/chosen").unwrap();
    let initrd_start_prop = chosen.properties().find(|p| p.name == "linux,initrd-start").unwrap();
    let initrd_end_prop = chosen.properties().find(|p| p.name == "linux,initrd-end").unwrap();
    let initrd_start = BE::read_uint(initrd_start_prop.data, 4) as usize;
    let initrd_end = BE::read_uint(initrd_end_prop.data, 4) as usize;
    let initrd_size = initrd_end - initrd_start;
    KERNEL_TABLE.map_to(VirtualAddress(0xFFFF_1000_4000_0000), PhysicalAddress(initrd_start), initrd_size).unwrap();
    let initrd = core::slice::from_raw_parts(0xFFFF_1000_4000_0000 as *const u8, initrd_size as usize);

    memory::init_main_heap(&mut KERNEL_TABLE);

    /*
        let mut node_path = String::from("/");
        let mut last_parents = 0;
        let trim_node_names = |s: &mut String, n| {
            for _ in 0..n {
                s.pop();
                'inner: loop {
                    if s.ends_with('/') || s == "" {
                        break 'inner;
                    } else {
                        s.pop();
                    }
                }
            }
        };
        for node in dt.nodes() {
            let depth_diff = node.parents as i8 - last_parents;
            last_parents = node.parents as i8;

            match depth_diff {
                x if x <= 0 => {
                    trim_node_names(&mut node_path, -x + 1);
                }
                1 => {}
                _ => panic!("fdt parser: sudden gain of parents"),
            }
            node_path.push_str(node.name);
            node_path.push('/');
            println!("{}, {}", node_path, node.offset);
            for prop in node.properties() {
                println!("\t{}: {:?}", prop.name, prop.data);
            }
        }
    */
    crate::main(Arch {
        device_tree: dt,
        initrd,
    });
}
