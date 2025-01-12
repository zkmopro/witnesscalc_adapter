#[link(name = "witnesscalc_authV2")]
extern "C" {
    pub fn witnesscalc_authV2(
        circuit_buffer: *const std::os::raw::c_char,
        circuit_size: libc::c_ulong,
        json_buffer: *const std::os::raw::c_char,
        json_size: libc::c_ulong,
        wtns_buffer: *mut std::os::raw::c_char,
        wtns_size: *mut libc::c_ulong,
        error_msg: *mut std::os::raw::c_char,
        error_msg_maxsize: libc::c_ulong,
    ) -> libc::c_int;
}

#[cfg(test)]
mod test {

    use std::{
        ffi::{CStr, CString},
        fs,
    };

    use crate::witnesscalc_authV2;

    #[test]
    fn test_witnesscalc_authV2_success() {
        let circuit_data = fs::read("./testdata/authV2.dat").unwrap();
        let circuit_buffer = circuit_data.as_ptr() as *const libc::c_char;
        let circuit_size = circuit_data.len() as libc::c_ulong;

        let json_data =
            fs::read("./testdata/authV2_input.json").expect("Couldn't read the input .json");
        let json_input = CString::new(json_data).unwrap();
        let json_size = json_input.as_bytes().len() as libc::c_ulong;

        let mut wtns_buffer = vec![0u8; 8 * 1024 * 1024]; // 8 MB buffer
        let mut wtns_size: libc::c_ulong = wtns_buffer.len() as libc::c_ulong;

        let mut error_msg = vec![0u8; 256];
        let error_msg_maxsize = error_msg.len() as libc::c_ulong;

        unsafe {
            let result = witnesscalc_authV2(
                circuit_buffer,
                circuit_size,
                json_input.as_ptr(),
                json_size,
                wtns_buffer.as_mut_ptr() as *mut _,
                &mut wtns_size as *mut _,
                error_msg.as_mut_ptr() as *mut _,
                error_msg_maxsize,
            );

            if result != 0 {
                let error_str = CStr::from_ptr(error_msg.as_ptr() as *const _)
                    .to_string_lossy()
                    .to_string();
                eprintln!("Error: {}", error_str);
            }

            assert!(result == 0);
        }
    }

    #[test]
    fn test_witnesscalc_authV2_failure() {
        let circuit_data = fs::read("./testdata/authV2.dat").unwrap();
        let circuit_buffer = circuit_data.as_ptr() as *const libc::c_char;
        let circuit_size = circuit_data.len() as libc::c_ulong;

        let json_data =
            fs::read("./testdata/authV2_input_wrong.json").expect("Couldn't read the input .json");
        let json_input = CString::new(json_data).unwrap();
        let json_size = json_input.as_bytes().len() as libc::c_ulong;

        let mut wtns_buffer = vec![0u8; 8 * 1024 * 1024]; // 8 MB buffer
        let mut wtns_size: libc::c_ulong = wtns_buffer.len() as libc::c_ulong;

        let mut error_msg = vec![0u8; 256];
        let error_msg_maxsize = error_msg.len() as libc::c_ulong;

        unsafe {
            let result = witnesscalc_authV2(
                circuit_buffer,
                circuit_size,
                json_input.as_ptr(),
                json_size,
                wtns_buffer.as_mut_ptr() as *mut _,
                &mut wtns_size as *mut _,
                error_msg.as_mut_ptr() as *mut _,
                error_msg_maxsize,
            );

            if result != 0 {
                let error_str = CStr::from_ptr(error_msg.as_ptr() as *const _)
                    .to_string_lossy()
                    .to_string();
                eprintln!("Error: {}", error_str);
            }

            assert!(result != 0);
        }
    }
}
