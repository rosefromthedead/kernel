use crate::{context::Context, vm::{Table, VirtualAddress}};

pub fn load_elf(file: &[u8], context: &mut Context) -> Result<(), goblin::error::Error> {
    let elf = goblin::elf::Elf::parse(file)?;
    for program_header in elf.program_headers {
        let vm_range = program_header.vm_range();

        if vm_range.start == 0 {
            break;
        }

        let size = vm_range.end - vm_range.start;
        context.table.alloc(VirtualAddress(vm_range.start), size).unwrap();
        let dest = unsafe {
            core::slice::from_raw_parts_mut(vm_range.start as *mut u8, size)
        };

        let src = &file[program_header.file_range()];
        dest.copy_from_slice(src);
    }

    context.set_entry_point(VirtualAddress(elf.entry as usize));

    Ok(())
}
