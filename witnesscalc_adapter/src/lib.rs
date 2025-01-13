pub use paste;

pub mod adapter;

#[macro_export]
macro_rules! witness {
    ($x: ident) => {
        $crate::paste::paste! {
                #[link(name = "witnesscalc_" [<$x>])]
                extern "C" {
                    pub fn [<witnesscalc_ $x>](
            circuit_buffer: *const std::os::raw::c_char,
            circuit_size: std::ffi::c_ulong,
            json_buffer: *const std::os::raw::c_char,
            json_size: std::ffi::c_ulong,
            wtns_buffer: *mut std::os::raw::c_char,
            wtns_size: *mut std::ffi::c_ulong,
            error_msg: *mut std::os::raw::c_char,
            error_msg_maxsize: std::ffi::c_ulong,
        ) -> std::ffi::c_int;
                }
            }
    };
}
