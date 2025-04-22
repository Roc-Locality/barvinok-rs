use crate::value::Value;
use std::fmt::Debug;
use std::ptr::NonNull;

use crate::{ContextRef, DimType, nonnull_or_alloc_error, printer::ISLPrint, space::Space};

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

macro_rules! qpolynomial_constructors {
    ($($func:ident),+ $(,)?) => {
        paste::paste! {
            $(
                pub fn [<new_ $func _on_domain>](space: Space<'a>) -> QuasiPolynomial<'a> {
                    let handle = unsafe {
                        barvinok_sys::[<isl_qpolynomial_ $func _on_domain>](space.handle.as_ptr())
                    };
                    let handle = nonnull_or_alloc_error(handle);
                    std::mem::forget(space);
                    QuasiPolynomial {
                        handle,
                        marker: std::marker::PhantomData,
                    }
                }
            )*
        }
    };
}

impl<'a> QuasiPolynomial<'a> {
    qpolynomial_constructors!(zero, one, infty, neginfty, nan);
    pub fn context_ref(&self) -> ContextRef<'a> {
        let ctx = unsafe { barvinok_sys::isl_qpolynomial_get_ctx(self.handle.as_ptr()) };
        let ctx = unsafe { NonNull::new_unchecked(ctx) };
        ContextRef(ctx, std::marker::PhantomData)
    }
    pub fn get_domain_space(&self) -> Space<'a> {
        let handle =
            unsafe { barvinok_sys::isl_qpolynomial_get_domain_space(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Space {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn get_space(&self) -> Space<'a> {
        let handle = unsafe { barvinok_sys::isl_qpolynomial_get_space(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Space {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn get_dim(&self, dim: DimType) -> usize {
        let dim = dim as barvinok_sys::isl_dim_type;
        unsafe { barvinok_sys::isl_qpolynomial_dim(self.handle.as_ptr(), dim) as usize }
    }
    pub fn new_val_on_domain(space: Space<'a>, value: Value<'a>) -> QuasiPolynomial<'a> {
        let handle = unsafe {
            barvinok_sys::isl_qpolynomial_val_on_domain(
                space.handle.as_ptr(),
                value.handle.as_ptr(),
            )
        };
        let handle = nonnull_or_alloc_error(handle);
        std::mem::forget(space);
        std::mem::forget(value);
        QuasiPolynomial {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn new_var_on_domain(
        space: Space<'a>,
        dim_type: DimType,
        pos: u32,
    ) -> Result<QuasiPolynomial<'a>, crate::Error> {
        let dim_size = space
            .get_dim(dim_type)
            .ok_or(crate::Error::VariablePositionOutOfBounds)?;
        if pos >= dim_size {
            return Err(crate::Error::VariablePositionOutOfBounds);
        }
        let handle = unsafe {
            barvinok_sys::isl_qpolynomial_var_on_domain(space.handle.as_ptr(), dim_type as u32, pos)
        };
        let handle = nonnull_or_alloc_error(handle);
        std::mem::forget(space);
        Ok(QuasiPolynomial {
            handle,
            marker: std::marker::PhantomData,
        })
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

    #[test]
    fn test_quasi_polynomial_get_space() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 1, 2);
        let qpoly = QuasiPolynomial::new_one_on_domain(space);
        let space2 = qpoly.get_space();
        println!("{:?}", space2);
        let space3 = qpoly.get_domain_space();
        println!("{:?}", space3);
    }

    #[test]
    fn test_quasi_polynomial_get_dim() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 1, 2);
        let qpoly = QuasiPolynomial::new_one_on_domain(space);
        let dim = qpoly.get_dim(DimType::Param);
        assert_eq!(dim, 1);
        let dim = qpoly.get_dim(DimType::Out);
        assert_eq!(dim, 1);
        let dim = qpoly.get_dim(DimType::In);
        assert_eq!(dim, 2);
    }

    #[test]
    fn test_quasi_polynomial_new_val_on_domain() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 1, 2);
        let value = Value::new_si(&ctx, 42);
        let qpoly = QuasiPolynomial::new_val_on_domain(space, value);
        assert_eq!(qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
        println!("{:?}", qpoly);
    }

    #[test]
    fn test_quasi_polynomial_new_var_on_domain() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 1, 2);
        let qpoly = QuasiPolynomial::new_var_on_domain(space.clone(), DimType::Param, 0).unwrap();
        assert_eq!(qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
        println!("{:?}", qpoly);
        let qpoly2 = QuasiPolynomial::new_var_on_domain(space, DimType::Out, 1).unwrap();
        assert_eq!(qpoly2.context_ref().0.as_ptr(), ctx.0.as_ptr());
        println!("{:?}", qpoly2);
    }

    #[test]
    #[should_panic(expected = "VariablePositionOutOfBounds")]
    fn test_invalid_var_pos() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 1, 2);
        QuasiPolynomial::new_var_on_domain(space, DimType::Param, 2).unwrap();
    }
}
