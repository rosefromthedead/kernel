use crate::{context::ActiveContext, vm::{Table, VirtualAddress}};

pub fn load_elf(file: &[u8], context: &mut ActiveContext) -> Result<(), goblin::error::Error> {
    let elf = goblin::elf::Elf::parse(file)?;
    let table = context.table();
    for program_header in elf.program_headers {
        let vm_range = program_header.vm_range();
        if vm_range.start == 0 {
            continue;
        }

        tracing::debug!(va = program_header.p_vaddr, size = program_header.p_memsz, "loading program header");

        let size = vm_range.end - vm_range.start;
        table.alloc(VirtualAddress(vm_range.start), size).unwrap();
        let dest = unsafe {
            core::slice::from_raw_parts_mut(vm_range.start as *mut u8, size)
        };

        let src = &file[program_header.file_range()];
        dest.copy_from_slice(src);
    }

    context.user_state.set_entry_point(VirtualAddress(elf.entry as usize));

    Ok(())
}
