#![no_std]
#![no_main]

mod uefi;

use r_efi::efi;
use utils::{handle_protocol, print, utf16_cstring};

use crate::uefi::{build_kernel_device_path, load_cmdline_options, load_kernel, start_kernel};

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

const PONG_GUID: efi::Guid =
    efi::Guid::from_fields(0x676E_6F6F, 0x4F50, 0x474E, 0x00, 0x00, &[0, 0, 0, 0, 0, 1]);

unsafe fn restore_text_output(st: *mut efi::SystemTable) {
    unsafe {
        let con_out = (*st).con_out;
        ((*con_out).reset)(con_out, efi::Boolean::FALSE);
        ((*con_out).clear_screen)(con_out);
    }
}

unsafe fn read_pong_result(st: *mut efi::SystemTable) -> Result<u8, efi::Status> {
    unsafe {
        let rt = (*st).runtime_services;
        let name = utf16_cstring::<12>("PONG_RESULT");
        let mut attrs = 0;
        let mut result = 0u8;
        let mut size = core::mem::size_of::<u8>();

        let status = ((*rt).get_variable)(
            name.as_ptr() as *mut u16,
            &PONG_GUID as *const _ as *mut _,
            &mut attrs,
            &mut size,
            &mut result as *mut _ as *mut _,
        );

        if status.is_error() {
            return Err(status);
        }

        if size != core::mem::size_of::<u8>() {
            return Err(efi::Status::BAD_BUFFER_SIZE);
        }

        Ok(result)
    }
}

unsafe fn clear_pong_result(st: *mut efi::SystemTable) -> Result<(), efi::Status> {
    unsafe {
        let rt = (*st).runtime_services;
        let name = utf16_cstring::<12>("PONG_RESULT");
        let status = ((*rt).set_variable)(
            name.as_ptr() as *mut u16,
            &PONG_GUID as *const _ as *mut _,
            0,
            0,
            core::ptr::null_mut(),
        );

        if status == efi::Status::SUCCESS || status == efi::Status::NOT_FOUND {
            return Ok(());
        }

        Err(status)
    }
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

        let game_path = build_kernel_device_path("\\game.efi", bs, device_handle)?;

        clear_pong_result(st)?;
        print(st, "got kernel device path... \r\n");

        let game_image = load_kernel(bs, h, game_path)?;
        let loaded_game = handle_protocol::<efi::protocols::loaded_image::Protocol>(
            bs,
            game_image,
            &efi::protocols::loaded_image::PROTOCOL_GUID as *const _ as *mut _,
        )?;

        utils::print(st, "loaded kernel image... \r\n");

        let cmdline = utf16_cstring::<128>("initrd=\\initrd.img console=tty0 loglevel=7");
        load_cmdline_options(&cmdline, loaded_game)?;

        print(st, "starting kernel image... \r\n");

        start_kernel(bs, game_image)?;
        restore_text_output(st);

        let pong_result = match read_pong_result(st) {
            Ok(result) => result,
            Err(status) => {
                print(st, "no result\r\n");
                return Err(status);
            }
        };
        clear_pong_result(st)?;

        if pong_result != 1 {
            print(st, "game lost\r\n");
            return Ok(());
        }

        let kernel_path = build_kernel_device_path("\\vmlinuz", bs, device_handle)?;
        let kernel_image = load_kernel(bs, h, kernel_path)?;
        let loaded_kernel = handle_protocol::<efi::protocols::loaded_image::Protocol>(
            bs,
            kernel_image,
            &efi::protocols::loaded_image::PROTOCOL_GUID as *const _ as *mut _,
        )?;

        load_cmdline_options(&cmdline, loaded_kernel)?;
        print(st, "starting vmlinuz... \r\n");
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
