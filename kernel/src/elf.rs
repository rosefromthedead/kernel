use crate::{arch::context::ActiveContext, fmt::ForceLowerHex, vm::{Table, VirtualAddress}};

pub fn load_elf(file: &[u8], context: &mut ActiveContext) -> Result<(), goblin::error::Error> {
    let _guard = tracing::debug_span!("loading elf file").entered();
    let elf = goblin::elf::Elf::parse(file)?;
    let table = unsafe { context.table() };
    for program_header in elf.program_headers.iter().filter(|h| h.p_type == 1) {
        let vm_range = program_header.vm_range();
        if vm_range.start == 0 {
            continue;
        }

        let start = VirtualAddress(vm_range.start);
        let size = vm_range.end - vm_range.start;
        tracing::debug!(va=?start, size=?ForceLowerHex(size), "loading program header");

        table.alloc(VirtualAddress(vm_range.start), size).unwrap();
        let dest = unsafe {
            core::slice::from_raw_parts_mut(vm_range.start as *mut u8, size)
        };

        let src = &file[program_header.file_range()];
        dest.copy_from_slice(src);
    }

    context.set_entry_point(VirtualAddress(elf.entry as usize));

    Ok(())
}
