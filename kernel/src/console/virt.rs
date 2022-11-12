pub fn virt_putchar(c: u8) {
    unsafe {
        let base = 0xFFFF_FF00_0000_0000 as *mut u32;
        let data_register = base;
        // let flag_register = base.offset(0x18);
        // wait for tx fifo not full
        // while flag_register.read_volatile() & 0x20 != 0 {}
        data_register.write_volatile(c as u32);
        // wait for tx fifo empty
        // while flag_register.read_volatile() & 0x80 == 0 {}
    }
}
