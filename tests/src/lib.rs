use std::ffi::{CStr, CString};

witnesscalc_adapter::witness!(multiplier2);

pub unsafe fn run_witnesscalc(
    circuit_buffer: *const i8,
    circuit_size: u64,
    json_input: CString,
    json_size: u64,
    mut wtns_buffer: Vec<u8>,
    mut wtns_size: u64,
    mut error_msg: Vec<u8>,
    error_msg_maxsize: u64,
) {
    let result = unsafe {
        witnesscalc_multiplier2(
            circuit_buffer,
            circuit_size,
            json_input.as_ptr(),
            json_size,
            wtns_buffer.as_mut_ptr() as *mut _,
            &mut wtns_size as *mut _,
            error_msg.as_mut_ptr() as *mut _,
            error_msg_maxsize,
        )
    };

    assert!(result == 0);
    let error_str = unsafe {
        CStr::from_ptr(error_msg.as_ptr() as *const _)
            .to_string_lossy()
            .to_string()
    };
    assert!(error_str.is_empty());
}

#[cfg(test)]
mod test {

    use std::{ffi::CString, fs};

    use crate::run_witnesscalc;

    #[test]
    fn test_witnesscalc_success() {
        let circuit_data = fs::read("./testdata/multiplier2.dat").unwrap();
        let circuit_buffer = circuit_data.as_ptr() as *const std::ffi::c_char;
        let circuit_size = circuit_data.len() as std::ffi::c_ulong;

        let json_data =
            fs::read("./testdata/multiplier2_input.json").expect("Couldn't read the input .json");
        let json_input = CString::new(json_data).unwrap();
        let json_size = json_input.as_bytes().len() as std::ffi::c_ulong;

        let wtns_buffer = vec![0u8; 8 * 1024 * 1024]; // 8 MB buffer
        let wtns_size: std::ffi::c_ulong = wtns_buffer.len() as std::ffi::c_ulong;

        let error_msg = vec![0u8; 256];
        let error_msg_maxsize = error_msg.len() as std::ffi::c_ulong;

        unsafe {
            run_witnesscalc(
                circuit_buffer,
                circuit_size,
                json_input,
                json_size,
                wtns_buffer,
                wtns_size,
                error_msg,
                error_msg_maxsize,
            )
        };
    }
}
