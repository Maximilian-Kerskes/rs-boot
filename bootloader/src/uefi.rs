use r_efi::efi;

use utils::utf16_cstring;

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

pub unsafe fn get_base_path(
    bs: *mut efi::BootServices,
    device_handle: efi::Handle,
) -> Result<*mut efi::protocols::device_path::Protocol, efi::Status> {
    unsafe {
        handle_protocol::<efi::protocols::device_path::Protocol>(
            bs,
            device_handle,
            &efi::protocols::device_path::PROTOCOL_GUID as *const _ as *mut _,
        )
    }
}

pub unsafe fn get_dpu(
    bs: *mut efi::BootServices,
) -> Result<*mut efi::protocols::device_path_utilities::Protocol, efi::Status> {
    unsafe {
        locate_protocol::<efi::protocols::device_path_utilities::Protocol>(
            bs,
            &efi::protocols::device_path_utilities::PROTOCOL_GUID as *const _ as *mut _,
        )
    }
}

pub unsafe fn get_dpft(
    bs: *mut efi::BootServices,
) -> Result<*mut efi::protocols::device_path_from_text::Protocol, efi::Status> {
    unsafe {
        locate_protocol::<efi::protocols::device_path_from_text::Protocol>(
            bs,
            &efi::protocols::device_path_from_text::PROTOCOL_GUID as *const _ as *mut _,
        )
    }
}

pub unsafe fn build_kernel_device_path(
    kernel_path: &str,
    bs: *mut efi::BootServices,
    device_handle: efi::Handle,
) -> Result<*mut efi::protocols::device_path::Protocol, efi::Status> {
    // build kernel device path:
    // 1. get root device path
    // 2. build kernel_device_path from kernel_path
    // 3. append kernel_device_path to root device path
    unsafe {
        let base_path = get_base_path(bs, device_handle)?;
        let dpu = get_dpu(bs)?;
        let dpft = get_dpft(bs)?;

        let kernel_rel_text = utf16_cstring::<32>(kernel_path);
        let kernel_rel_path = ((*dpft).convert_text_to_device_path)(kernel_rel_text.as_ptr());
        if kernel_rel_path.is_null() {
            return Err(efi::Status::INVALID_PARAMETER);
        }

        let kernel_path = ((*dpu).append_device_path)(base_path, kernel_rel_path);
        if kernel_path.is_null() {
            return Err(efi::Status::OUT_OF_RESOURCES);
        }

        Ok(kernel_path)
    }
}

pub unsafe fn load_kernel(
    bs: *mut efi::BootServices,
    h: efi::Handle,
    kernel_path: *mut efi::protocols::device_path::Protocol,
) -> Result<efi::Handle, efi::Status> {
    unsafe {
        let mut kernel_image: efi::Handle = core::ptr::null_mut();

        let result = ((*bs).load_image)(
            efi::Boolean::FALSE,
            h,
            kernel_path,
            core::ptr::null_mut(),
            0,
            &mut kernel_image as *mut _,
        );
        if result.is_error() {
            return Err(result);
        }

        Ok(kernel_image)
    }
}

pub unsafe fn load_cmdline_options(
    cmdline: &[u16],
    loaded_kernel: *mut efi::protocols::loaded_image::Protocol,
) -> Result<(), efi::Status> {
    unsafe {
        (*loaded_kernel).load_options = cmdline.as_ptr() as *mut core::ffi::c_void;
        (*loaded_kernel).load_options_size = core::mem::size_of_val(cmdline) as u32;
        Ok(())
    }
}

pub unsafe fn start_kernel(
    bs: *mut efi::BootServices,
    kernel_image: efi::Handle,
) -> Result<(), efi::Status> {
    unsafe {
        let result =
            ((*bs).start_image)(kernel_image, core::ptr::null_mut(), core::ptr::null_mut());
        if result.is_error() {
            return Err(result);
        }
        Ok(())
    }
}
