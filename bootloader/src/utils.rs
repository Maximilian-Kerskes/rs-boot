use r_efi::efi;

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

pub fn print(st: *mut efi::SystemTable, s: &str) {
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
