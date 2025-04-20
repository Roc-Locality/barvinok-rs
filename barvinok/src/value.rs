use std::ptr::NonNull;

use num_traits::PrimInt;

use crate::{Context, nonnull_or_alloc_error, stat::Flag};

#[repr(transparent)]
pub struct Value<'a> {
    handle: NonNull<barvinok_sys::isl_val>,
    marker: std::marker::PhantomData<&'a ()>,
}

macro_rules! isl_val_new {
    ($name:ident, $func:ident $(, $arg_name:ident : $arg_ty:ty)*) => {
        pub fn $name(ctx: &'a Context $(, $arg_name: $arg_ty)*) -> Self {
            let handle = unsafe { barvinok_sys::$func(ctx.0.as_ptr() $(, $arg_name)*) };
            let handle = nonnull_or_alloc_error(handle);
            Self {
                handle,
                marker: std::marker::PhantomData,
            }
        }
    };
}

impl<'a> Value<'a> {
    isl_val_new!(new_zero, isl_val_zero);
    isl_val_new!(new_one, isl_val_one);
    isl_val_new!(new_negone, isl_val_negone);
    isl_val_new!(new_nan, isl_val_nan);
    isl_val_new!(new_infty, isl_val_infty);
    isl_val_new!(new_neg_infty, isl_val_neginfty);
    isl_val_new!(new_si, isl_val_int_from_si, value: i64);
    isl_val_new!(new_ui, isl_val_int_from_ui, value: u64);

    pub fn new_chunks<T: PrimInt>(ctx: &'a Context, value: &[T]) -> Self {
        let handle = unsafe {
            barvinok_sys::isl_val_int_from_chunks(
                ctx.0.as_ptr(),
                value.len(),
                std::mem::size_of::<T>(),
                value.as_ptr() as *const std::ffi::c_void,
            )
        };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }

    pub fn numerator(&self) -> i64 {
        unsafe { barvinok_sys::isl_val_get_num_si(self.handle.as_ptr()) }
    }

    pub fn denominator(&self) -> i64 {
        unsafe { barvinok_sys::isl_val_get_den_si(self.handle.as_ptr()) }
    }

    pub fn denominator_value(&self) -> Self {
        let handle = unsafe { barvinok_sys::isl_val_get_den_val(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }

    pub fn to_f64(&self) -> f64 {
        unsafe { barvinok_sys::isl_val_get_d(self.handle.as_ptr()) }
    }

    pub fn abs_eq(&self, other: &Self) -> bool {
        let flag =
            unsafe { barvinok_sys::isl_val_abs_eq(self.handle.as_ptr(), other.handle.as_ptr()) };
        let flag = Flag::from_isl_bool(flag);
        matches!(flag, Flag::True)
    }
}

impl Drop for Value<'_> {
    fn drop(&mut self) {
        unsafe { barvinok_sys::isl_val_free(self.handle.as_ptr()) };
    }
}

impl Clone for Value<'_> {
    fn clone(&self) -> Self {
        let handle = unsafe { barvinok_sys::isl_val_copy(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl From<Value<'_>> for f64 {
    fn from(value: Value<'_>) -> Self {
        value.to_f64()
    }
}

macro_rules! impl_cmp_method {
    ($method:ident, $isl_fn:ident) => {
        fn $method(&self, other: &Self) -> bool {
            let flag =
                unsafe { barvinok_sys::$isl_fn(self.handle.as_ptr(), other.handle.as_ptr()) };
            matches!(Flag::from_isl_bool(flag), Flag::True)
        }
    };
}

impl PartialEq for Value<'_> {
    impl_cmp_method!(eq, isl_val_eq);
    impl_cmp_method!(ne, isl_val_ne);
}

impl PartialOrd for Value<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.lt(other) {
            Some(std::cmp::Ordering::Less)
        } else if self.eq(other) {
            Some(std::cmp::Ordering::Equal)
        } else if self.gt(other) {
            Some(std::cmp::Ordering::Greater)
        } else {
            None
        }
    }

    impl_cmp_method!(ge, isl_val_ge);
    impl_cmp_method!(le, isl_val_le);
    impl_cmp_method!(gt, isl_val_gt);
    impl_cmp_method!(lt, isl_val_lt);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_value() {
        let ctx = Context::new();
        let val = Value::new_si(&ctx, 42);
        assert_eq!(val.numerator(), 42);
        assert_eq!(val.denominator(), 1);
        assert_eq!(val.to_f64(), 42.0);
    }

    #[test]
    fn test_value_clone() {
        let ctx = Context::new();
        let val = Value::new_si(&ctx, 42);
        let val_clone = val.clone();
        assert_eq!(val_clone.numerator(), 42);
        assert_eq!(val_clone.denominator(), 1);
        assert_eq!(val_clone.to_f64(), 42.0);
    }

    #[test]
    fn test_value_chunks() {
        let ctx = Context::new();
        let val = Value::new_chunks(&ctx, &[0, 2, 2]);
        assert!((val.to_f64() - 2.0f64.powi(33) - 2.0f64.powi(65)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_value_abs_eq() {
        let ctx = Context::new();
        let val1 = Value::new_si(&ctx, 42);
        let val2 = Value::new_si(&ctx, -42);
        assert!(val1.abs_eq(&val2));
    }

    #[test]
    fn test_value_cmp() {
        let ctx = Context::new();
        let val1 = Value::new_si(&ctx, 42);
        let val2 = Value::new_si(&ctx, 43);
        assert!(val1 < val2);
        assert!(val1 <= val2);
        assert!(val2 > val1);
        assert!(val2 >= val1);
        assert!(val1 != val2);
        assert!(val1 == val1);
        assert!(val2 == val2);
    }
}
