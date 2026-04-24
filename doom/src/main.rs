#![no_std]
#![no_main]

use r_efi::efi;

unsafe extern "C" {
    fn doomgeneric_Create(argc: i32, argv: *const *const u8);
    fn doomgeneric_Tick();
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn DG_DrawFrame() {
    // noop
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn efi_main(_h: efi::Handle, _st: *mut efi::SystemTable) -> efi::Status {
    unsafe {
        doomgeneric_Create(0, core::ptr::null());
    }
    loop {
        unsafe {
            doomgeneric_Tick();
        }
    }
}
