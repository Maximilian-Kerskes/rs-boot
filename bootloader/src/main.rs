#![no_std]
#![no_main]

mod uefi;
mod utils;

use r_efi::efi;
use utils::print;

use crate::uefi::{
    build_kernel_device_path, handle_protocol, load_cmdline_options, load_kernel, start_kernel,
};
use crate::utils::utf16_cstring;

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

unsafe fn try_efi_main(h: efi::Handle, st: *mut efi::SystemTable) -> Result<(), efi::Status> {
    unsafe {
        print(st, "Init... \r\n");

        let bs = (*st).boot_services;

        let loaded_image = handle_protocol::<efi::protocols::loaded_image::Protocol>(
            bs,
            h,
            &efi::protocols::loaded_image::PROTOCOL_GUID as *const _ as *mut _,
        )?;

        let device_handle = (*loaded_image).device_handle;

        let kernel_path = build_kernel_device_path("\\vmlinuz", bs, device_handle)?;

        print(st, "got kernel device path... \r\n");

        let kernel_image = load_kernel(bs, h, kernel_path)?;
        let loaded_kernel = handle_protocol::<efi::protocols::loaded_image::Protocol>(
            bs,
            kernel_image,
            &efi::protocols::loaded_image::PROTOCOL_GUID as *const _ as *mut _,
        )?;

        utils::print(st, "loaded kernel image... \r\n");

        let cmdline = utf16_cstring::<128>("initrd=\\initrd.img console=tty0 loglevel=7");
        load_cmdline_options(&cmdline, loaded_kernel)?;

        print(st, "starting kernel image... \r\n");

        start_kernel(bs, kernel_image)?;
    }

    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn efi_main(h: efi::Handle, st: *mut efi::SystemTable) -> efi::Status {
    match unsafe { try_efi_main(h, st) } {
        Ok(()) => efi::Status::SUCCESS,
        Err(status) => status,
    }
}
