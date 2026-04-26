#![no_std]

use r_efi::efi;

// UEFI Utilities
pub unsafe fn locate_protocol<P>(
    bs: *mut efi::BootServices,
    guid: *mut efi::Guid,
) -> Result<*mut P, efi::Status> {
    unsafe {
        let mut protocol: *mut P = core::ptr::null_mut();

        let status =
            ((*bs).locate_protocol)(guid, core::ptr::null_mut(), &mut protocol as *mut _ as *mut _);

        if status.is_error() {
            return Err(status);
        }

        Ok(protocol)
    }
}
pub unsafe fn handle_protocol<P>(
    bs: *mut efi::BootServices,
    handle: efi::Handle,
    guid: *mut efi::Guid,
) -> Result<*mut P, efi::Status> {
    unsafe {
        let mut protocol: *mut P = core::ptr::null_mut();

        let status = ((*bs).handle_protocol)(handle, guid, &mut protocol as *mut _ as *mut _);

        if status.is_error() {
            return Err(status);
        }

        Ok(protocol)
    }
}

pub fn utf16_cstring<const N: usize>(s: &str) -> [u16; N] {
    let mut buf = [0u16; N];
    let mut len = 0;

    for (i, c) in s.encode_utf16().enumerate() {
        buf[i] = c;
        len = i + 1;
    }
    buf[len] = 0;
    buf
}

pub unsafe fn print(st: *mut efi::SystemTable, s: &str) {
    let buf = utf16_cstring::<32>(s);

    unsafe {
        (((*(*st).con_out).output_string)((*st).con_out, buf.as_ptr() as *mut _));
    }
}
