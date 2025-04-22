use std::fmt::Debug;
use std::ptr::NonNull;

use crate::{ContextRef, nonnull_or_alloc_error, printer::ISLPrint, space::Space};

#[repr(transparent)]
pub struct QuasiPolynomial<'a> {
    handle: NonNull<barvinok_sys::isl_qpolynomial>,
    marker: std::marker::PhantomData<&'a ()>,
}

#[repr(transparent)]
pub struct PiecewiseQuasiPolynomial<'a> {
    handle: NonNull<barvinok_sys::isl_pw_qpolynomial>,
    marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> QuasiPolynomial<'a> {
    pub fn new_zero_on_domain(space: Space<'a>) -> Self {
        let handle = unsafe { barvinok_sys::isl_qpolynomial_zero_on_domain(space.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        std::mem::forget(space);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn context_ref(&self) -> ContextRef<'a> {
        let ctx = unsafe { barvinok_sys::isl_qpolynomial_get_ctx(self.handle.as_ptr()) };
        let ctx = unsafe { NonNull::new_unchecked(ctx) };
        ContextRef(ctx, std::marker::PhantomData)
    }
}

impl Clone for QuasiPolynomial<'_> {
    fn clone(&self) -> Self {
        let handle = unsafe { barvinok_sys::isl_qpolynomial_copy(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl Drop for QuasiPolynomial<'_> {
    fn drop(&mut self) {
        unsafe { barvinok_sys::isl_qpolynomial_free(self.handle.as_ptr()) };
    }
}

impl<'a> ISLPrint<'a> for QuasiPolynomial<'a> {
    type Handle = barvinok_sys::isl_qpolynomial;

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
        unsafe { barvinok_sys::isl_printer_print_qpolynomial(printer, handle) }
    }
}

impl Debug for QuasiPolynomial<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wrapper = crate::printer::FmtWrapper::new(self);
        Debug::fmt(&wrapper, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_quasi_polynomial_create() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 1, 2);
        let qpoly = QuasiPolynomial::new_zero_on_domain(space);
        assert_eq!(qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
        println!("{:?}", qpoly);
    }
}
