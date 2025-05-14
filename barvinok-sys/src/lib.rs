#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(non_upper_case_globals)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::ptr_offset_with_cast)]
#![no_std]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

unsafe extern "C" {
    pub static barvinok_options_args: isl_args;
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::{isl_ctx_alloc, isl_ctx_free};

    #[test]
    fn it_creates_isl_context() {
        unsafe {
            let context = isl_ctx_alloc();
            isl_ctx_free(context);
        }
    }

    #[test]
    fn it_checks_barvinok_options_args() {
        unsafe {
            std::println!(
                "barvinok_options_args: {:?}",
                super::barvinok_options_args.options_size
            );
        }
    }
}
