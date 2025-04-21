use std::fmt::Debug;
use std::ptr::NonNull;

use crate::{Context, ContextRef, nonnull_or_alloc_error, printer::ISLPrint};

#[repr(transparent)]
pub struct Space<'a> {
    pub(crate) handle: NonNull<barvinok_sys::isl_space>,
    pub(crate) marker: std::marker::PhantomData<*mut &'a ()>,
}

impl<'a> ISLPrint<'a> for Space<'a> {
    type Handle = barvinok_sys::isl_space;

    fn context(&self) -> ContextRef<'a> {
        self.context_ref()
    }

    fn handle(&self) -> *mut Self::Handle {
        self.handle.as_ptr()
    }

    unsafe fn isl_printer_print(
        printer: *mut barvinok_sys::isl_printer,
        handle: *mut Self::Handle,
    ) -> *mut barvinok_sys::isl_printer {
        unsafe { barvinok_sys::isl_printer_print_space(printer, handle) }
    }
}

impl Debug for Space<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wrapper = crate::printer::FmtWrapper::new(self);
        Debug::fmt(&wrapper, f)
    }
}

macro_rules! space_constructor {
    // Pattern with additional arguments
    ($name:ident, $sys_fn:ident $(, $arg:ident : $arg_ty:ty)*) => {
        pub fn $name(ctx: &'a Context $(, $arg: $arg_ty)*) -> Self {
            let handle = unsafe {
                barvinok_sys::$sys_fn(ctx.0.as_ptr() $(, $arg)*)
            };
            let handle = nonnull_or_alloc_error(handle);
            Self {
                handle,
                marker: std::marker::PhantomData,
            }
        }
    };
}

impl<'a> Space<'a> {
    space_constructor!(new,        isl_space_alloc,    num_params: u32, num_inputs: u32, num_outputs: u32);
    space_constructor!(new_set,    isl_space_set_alloc,    num_params: u32, num_dims: u32);
    space_constructor!(new_params, isl_space_params_alloc, num_params: u32);
    space_constructor!(new_unit, isl_space_unit);

    pub fn context_ref(&self) -> ContextRef<'a> {
        ContextRef(
            unsafe {
                NonNull::new_unchecked(barvinok_sys::isl_space_get_ctx(self.handle.as_ptr()))
            },
            std::marker::PhantomData,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_space_creation() {
        let ctx = Context::new();
        let space = Space::new(&ctx, 2, 4, 3);
        println!("{:?}", space);
    }

    #[test]
    fn test_space_params() {
        let ctx = Context::new();
        let space = Space::new_params(&ctx, 2);
        println!("{:?}", space);
    }

    #[test]
    fn test_space_unit() {
        let ctx = Context::new();
        let space = Space::new_unit(&ctx);
        println!("{:?}", space);
    }

    #[test]
    fn test_space_set() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 2, 3);
        println!("{:?}", space);
    }
}
