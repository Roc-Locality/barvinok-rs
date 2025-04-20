use std::ptr::NonNull;

use num_traits::PrimInt;

use crate::{Context, nonnull_or_alloc_error};

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
}
