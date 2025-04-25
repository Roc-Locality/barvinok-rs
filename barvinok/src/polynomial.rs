use crate::{impl_isl_print, stat::isl_bool_to_optional_bool, value::Value};
use std::ptr::NonNull;

use crate::{ContextRef, DimType, nonnull_or_alloc_error, space::Space};
use std::mem::ManuallyDrop;

#[repr(transparent)]
pub struct QuasiPolynomial<'a> {
    handle: NonNull<barvinok_sys::isl_qpolynomial>,
    marker: std::marker::PhantomData<&'a ()>,
}

#[repr(transparent)]
pub struct PiecewiseQuasiPolynomial<'a> {
    pub(crate) handle: NonNull<barvinok_sys::isl_pw_qpolynomial>,
    pub(crate) marker: std::marker::PhantomData<&'a ()>,
}

macro_rules! qpolynomial_constructors {
    ($($func:ident),+ $(,)?) => {
        paste::paste! {
            $(
                pub fn [<new_ $func _on_domain>](space: Space<'a>) -> QuasiPolynomial<'a> {
                    let space = ManuallyDrop::new(space);
                    let handle = unsafe {
                        barvinok_sys::[<isl_qpolynomial_ $func _on_domain>](space.handle.as_ptr())
                    };
                    let handle = nonnull_or_alloc_error(handle);
                    QuasiPolynomial {
                        handle,
                        marker: std::marker::PhantomData,
                    }
                }
            )*
        }
    };
}

macro_rules! qpolynomial_flag {
    ($name:ident, $sys_fn:ident) => {
        pub fn $name(&self) -> Option<bool> {
            let flag = unsafe { barvinok_sys::$sys_fn(self.handle.as_ptr()) };
            isl_bool_to_optional_bool(flag)
        }
    };
}

macro_rules! impl_unary_op_qpolynomial_inline {
    ($vis:vis $method:ident, $isl_fn:ident) => {
        $vis fn $method(self) -> Self {
            let this = ManuallyDrop::new(self);
            let handle = unsafe { barvinok_sys::$isl_fn(this.handle.as_ptr()) };
            let handle = nonnull_or_alloc_error(handle);
            QuasiPolynomial {
                handle,
                marker: std::marker::PhantomData,
            }
        }
    };
}

macro_rules! impl_bin_op_qpolynomial_inline {
    ($vis:vis $method:ident, $isl_fn:ident, $other:ty) => {
        $vis fn $method(self, other: $other) -> Self {
            let this = ManuallyDrop::new(self);
            let other = ManuallyDrop::new(other);
            let handle =
                unsafe { barvinok_sys::$isl_fn(this.handle.as_ptr(), other.handle.as_ptr()) };
            let handle = nonnull_or_alloc_error(handle);
            QuasiPolynomial {
                handle,
                marker: std::marker::PhantomData,
            }
        }
    };
    (trivial $vis:vis $method:ident, $isl_fn:ident, $other:ty) => {
        $vis fn $method(self, other: $other) -> Self {
            let this = ManuallyDrop::new(self);
            let handle =
                unsafe { barvinok_sys::$isl_fn(this.handle.as_ptr(), other) };
            let handle = nonnull_or_alloc_error(handle);
            QuasiPolynomial {
                handle,
                marker: std::marker::PhantomData,
            }
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
        let space = ManuallyDrop::new(space);
        let value = ManuallyDrop::new(value);
        let handle = unsafe {
            barvinok_sys::isl_qpolynomial_val_on_domain(
                space.handle.as_ptr(),
                value.handle.as_ptr(),
            )
        };
        let handle = nonnull_or_alloc_error(handle);
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
        let space = ManuallyDrop::new(space);
        let handle = unsafe {
            barvinok_sys::isl_qpolynomial_var_on_domain(space.handle.as_ptr(), dim_type as u32, pos)
        };
        let handle = nonnull_or_alloc_error(handle);
        Ok(QuasiPolynomial {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn get_constant_value(&self) -> Option<Value<'a>> {
        let handle =
            unsafe { barvinok_sys::isl_qpolynomial_get_constant_val(self.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| Value {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    qpolynomial_flag!(is_zero, isl_qpolynomial_is_zero);
    qpolynomial_flag!(is_infty, isl_qpolynomial_is_infty);
    qpolynomial_flag!(is_neginfty, isl_qpolynomial_is_neginfty);
    qpolynomial_flag!(is_nan, isl_qpolynomial_is_nan);
    impl_bin_op_qpolynomial_inline!(trivial pub pow, isl_qpolynomial_pow, u32);
    impl_bin_op_qpolynomial_inline!(pub scale_up_val, isl_qpolynomial_scale_val, Value<'a>);
    impl_bin_op_qpolynomial_inline!(pub scale_down_val, isl_qpolynomial_scale_down_val, Value<'a>);
    impl_unary_op_qpolynomial_inline!(pub domain_reverse, isl_qpolynomial_domain_reverse);
    impl_unary_op_qpolynomial_inline!(pub homogenize, isl_qpolynomial_homogenize);
    pub fn plain_equal(&self, other: &QuasiPolynomial<'a>) -> Option<bool> {
        let flag = unsafe {
            barvinok_sys::isl_qpolynomial_plain_is_equal(
                self.handle.as_ptr(),
                other.handle.as_ptr(),
            )
        };
        isl_bool_to_optional_bool(flag)
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

macro_rules! impl_bin_op_qpolynomial {
    ($trait:ident, $method:ident, $isl_fn:ident) => {
        impl<'a> std::ops::$trait for QuasiPolynomial<'a> {
            type Output = QuasiPolynomial<'a>;
            impl_bin_op_qpolynomial_inline!($method, $isl_fn, Self::Output);
        }
    };
}

impl_bin_op_qpolynomial!(Add, add, isl_qpolynomial_add);
impl_bin_op_qpolynomial!(Sub, sub, isl_qpolynomial_sub);
impl_bin_op_qpolynomial!(Mul, mul, isl_qpolynomial_mul);

impl<'a> PiecewiseQuasiPolynomial<'a> {
    pub fn context_ref(&self) -> ContextRef<'a> {
        let ctx = unsafe { barvinok_sys::isl_pw_qpolynomial_get_ctx(self.handle.as_ptr()) };
        let ctx = unsafe { NonNull::new_unchecked(ctx) };
        ContextRef(ctx, std::marker::PhantomData)
    }
    pub fn get_space(&self) -> Space<'a> {
        let handle = unsafe { barvinok_sys::isl_pw_qpolynomial_get_space(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Space {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn involves_nan(&self) -> Option<bool> {
        let flag = unsafe { barvinok_sys::isl_pw_qpolynomial_involves_nan(self.handle.as_ptr()) };
        isl_bool_to_optional_bool(flag)
    }
    pub fn plain_equal(&self, other: &PiecewiseQuasiPolynomial<'a>) -> Option<bool> {
        let flag = unsafe {
            barvinok_sys::isl_pw_qpolynomial_plain_is_equal(
                self.handle.as_ptr(),
                other.handle.as_ptr(),
            )
        };
        isl_bool_to_optional_bool(flag)
    }
    pub fn new_zero(space: Space<'a>) -> PiecewiseQuasiPolynomial<'a> {
        let space = ManuallyDrop::new(space);
        let handle = unsafe { barvinok_sys::isl_pw_qpolynomial_zero(space.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        PiecewiseQuasiPolynomial {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn from_qpolynomial(polynomial: QuasiPolynomial<'a>) -> PiecewiseQuasiPolynomial<'a> {
        let polynomial = ManuallyDrop::new(polynomial);
        let handle = unsafe {
            barvinok_sys::isl_pw_qpolynomial_from_qpolynomial(polynomial.handle.as_ptr())
        };
        let handle = nonnull_or_alloc_error(handle);
        PiecewiseQuasiPolynomial {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl Clone for PiecewiseQuasiPolynomial<'_> {
    fn clone(&self) -> Self {
        let handle = unsafe { barvinok_sys::isl_pw_qpolynomial_copy(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}
impl Drop for PiecewiseQuasiPolynomial<'_> {
    fn drop(&mut self) {
        unsafe { barvinok_sys::isl_pw_qpolynomial_free(self.handle.as_ptr()) };
    }
}

impl_isl_print!(
    QuasiPolynomial,
    isl_qpolynomial,
    isl_printer_print_qpolynomial
);
impl_isl_print!(
    PiecewiseQuasiPolynomial,
    isl_pw_qpolynomial,
    isl_printer_print_pw_qpolynomial
);

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

    #[test]
    fn test_quasi_polynomial_add() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 1, 2);
        let qpoly1 = QuasiPolynomial::new_one_on_domain(space.clone());
        let qpoly2 = QuasiPolynomial::new_zero_on_domain(space);
        let qpoly3 = qpoly1 + qpoly2;
        assert_eq!(qpoly3.context_ref().0.as_ptr(), ctx.0.as_ptr());
        println!("{:?}", qpoly3);
    }

    #[test]
    fn test_quasi_polynomial_sub() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 1, 2);
        let qpoly1 = QuasiPolynomial::new_zero_on_domain(space.clone());
        let qpoly2 = QuasiPolynomial::new_one_on_domain(space);
        let qpoly3 = qpoly1 - qpoly2;
        assert_eq!(qpoly3.context_ref().0.as_ptr(), ctx.0.as_ptr());
        println!("{:?}", qpoly3);
        let val = qpoly3.get_constant_value();
        assert!(val.is_some());
        assert_eq!(val.unwrap().to_f64(), -1.0);
    }

    #[test]
    fn test_pw_qpolynomial_zero() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 1, 2);
        let pw_qpoly = PiecewiseQuasiPolynomial::new_zero(space);
        assert_eq!(pw_qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
        println!("{:?}", pw_qpoly);
    }

    #[test]
    fn test_pw_qpolynomial_from_qpolynomial() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 1, 2);
        let qpoly = QuasiPolynomial::new_one_on_domain(space);
        let pw_qpoly = PiecewiseQuasiPolynomial::from_qpolynomial(qpoly);
        assert_eq!(pw_qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
        println!("{:?}", pw_qpoly);
    }
}
