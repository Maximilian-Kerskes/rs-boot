#![no_std]
#![no_main]

use r_efi::efi;

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn utf16_cstring<const N: usize>(s: &str) -> [u16; N] {
    let mut buf = [0u16; N];
    let mut len = 0;

    for (i, c) in s.encode_utf16().enumerate() {
        buf[i] = c;
        len = i + 1;
    }
    buf[len] = 0;
    buf
}

fn print(st: *mut efi::SystemTable, s: &str) {
    let mut buf = [0u16; 256];

    let mut len = 0;
    for (i, c) in s.encode_utf16().enumerate() {
        buf[i] = c;
        len = i + 1;
    }

    buf[len] = 0;

    unsafe {
        (((*(*st).con_out).output_string)((*st).con_out, buf.as_ptr() as *mut _));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn efi_main(h: efi::Handle, st: *mut efi::SystemTable) -> efi::Status {
    unsafe {
        print(st, "Init... \r\n");

        let bs = (*st).boot_services;

        let mut loaded_image: *mut efi::protocols::loaded_image::Protocol = core::ptr::null_mut();
        let result = ((*bs).handle_protocol)(
            h,
            &efi::protocols::loaded_image::PROTOCOL_GUID as *const _ as *mut _,
            &mut loaded_image as *mut _ as *mut _,
        );
        if result.is_error() {
            return result;
        }

        let device_handle = (*loaded_image).device_handle;

        let mut fs: *mut efi::protocols::simple_file_system::Protocol = core::ptr::null_mut();
        let result = ((*bs).handle_protocol)(
            device_handle,
            &efi::protocols::simple_file_system::PROTOCOL_GUID as *const _ as *mut _,
            &mut fs as *mut _ as *mut _,
        );
        if result.is_error() {
            return result;
        }

        print(st, "got fs... \r\n");

        let mut root: *mut efi::protocols::file::Protocol = core::ptr::null_mut();

        let result = ((*fs).open_volume)(fs, &mut root);
        if result.is_error() {
            return result;
        }

        print(st, "got root dir... \r\n");

        let mut file: *mut efi::protocols::file::Protocol = core::ptr::null_mut();
        let mut path = [0u16; 16];

        let mut len = 0;
        for (i, c) in "\\vmlinuz".encode_utf16().enumerate() {
            path[i] = c;
            len = i + 1;
        }

        path[len] = 0;

        let result = ((*root).open)(
            root,
            &mut file,
            path.as_ptr() as *mut _,
            efi::protocols::file::MODE_READ,
            0,
        );
        if result.is_error() {
            return result;
        }

        print(st, "got vmlinuz... \r\n");

        // this entire part feels shitty as hell
        // abstract into helper
        // builds h device path, builds kernel device path, appends kernel device path to h device path
        let mut base_path: *mut efi::protocols::device_path::Protocol = core::ptr::null_mut();
        let result = ((*bs).handle_protocol)(
            device_handle,
            &efi::protocols::device_path::PROTOCOL_GUID as *const _ as *mut _,
            &mut base_path as *mut _ as *mut _,
        );
        if result.is_error() {
            return result;
        }

        let mut dpu: *mut efi::protocols::device_path_utilities::Protocol = core::ptr::null_mut();
        let result = ((*bs).locate_protocol)(
            &efi::protocols::device_path_utilities::PROTOCOL_GUID as *const _ as *mut _,
            core::ptr::null_mut(),
            &mut dpu as *mut _ as *mut _,
        );
        if result.is_error() {
            return result;
        }

        let mut dpft: *mut efi::protocols::device_path_from_text::Protocol = core::ptr::null_mut();
        let result = ((*bs).locate_protocol)(
            &efi::protocols::device_path_from_text::PROTOCOL_GUID as *const _ as *mut _,
            core::ptr::null_mut(),
            &mut dpft as *mut _ as *mut _,
        );
        if result.is_error() {
            return result;
        }

        let kernel_rel_text = utf16_cstring::<32>("\\vmlinuz");
        let kernel_rel_path = ((*dpft).convert_text_to_device_path)(kernel_rel_text.as_ptr());
        if kernel_rel_path.is_null() {
            return efi::Status::INVALID_PARAMETER;
        }

        let kernel_path = ((*dpu).append_device_path)(base_path, kernel_rel_path);
        if kernel_path.is_null() {
            return efi::Status::OUT_OF_RESOURCES;
        }

        print(st, "got kernel device path... \r\n");

        let mut kernel_image: efi::Handle = core::ptr::null_mut();

        let result = ((*bs).load_image)(
            efi::Boolean::FALSE,
            h,
            kernel_path,
            core::ptr::null_mut(),
            0,
            &mut kernel_image,
        );

        if result.is_error() {
            return result;
        }
        print(st, "loaded kernel image... \r\n");

        let mut loaded_kernel: *mut efi::protocols::loaded_image::Protocol = core::ptr::null_mut();
        let result = ((*bs).handle_protocol)(
            kernel_image,
            &efi::protocols::loaded_image::PROTOCOL_GUID as *const _ as *mut _,
            &mut loaded_kernel as *mut _ as *mut _,
        );

        if result.is_error() {
            return result;
        }

        let cmdline_str = "initrd=\\initrd.img console=tty0 loglevel=7";
        let cmdline = utf16_cstring::<128>(cmdline_str);
        let cmdline_len = cmdline_str.encode_utf16().count() + 1;
        (*loaded_kernel).load_options = cmdline.as_ptr() as *mut core::ffi::c_void;
        (*loaded_kernel).load_options_size = (cmdline_len * core::mem::size_of::<u16>()) as u32;

        print(st, "starting kernel image... \r\n");

        let result =
            ((*bs).start_image)(kernel_image, core::ptr::null_mut(), core::ptr::null_mut());

        if result.is_error() {
            return result;
        }
    }

    efi::Status::SUCCESS
}
