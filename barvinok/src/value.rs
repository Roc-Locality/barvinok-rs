use std::mem::ManuallyDrop;

use num_traits::PrimInt;

use crate::{ContextRef, impl_isl_handle, nonnull_or_alloc_error};

impl_isl_handle!(Value, val);

include!(concat!(env!("OUT_DIR"), "/generated/value.rs"));

impl<'a> Value<'a> {
    pub fn new_chunks<T: PrimInt>(ctx: ContextRef<'a>, value: &[T]) -> Self {
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

    pub fn to_f64(&self) -> f64 {
        unsafe { barvinok_sys::isl_val_get_d(self.handle.as_ptr()) }
    }

    pub fn cmp_si(&self, value: i64) -> Option<std::cmp::Ordering> {
        if !self.is_nan().ok()? {
            let int_val = unsafe { barvinok_sys::isl_val_cmp_si(self.handle.as_ptr(), value) };
            Some(int_val.cmp(&0))
        } else {
            None
        }
    }

    pub fn checked_exp2(self) -> crate::Result<Self> {
        if !self.is_int()? {
            return Err(crate::Error::NonIntegralValue);
        }
        self.pow2()
    }

    pub fn checked_rem(self, other: Self) -> crate::Result<Self> {
        if !self.is_int()? || !other.is_int()? {
            return Err(crate::Error::NonIntegralValue);
        }
        self.modulo(other)
    }

    pub fn checked_gcd(self, other: Self) -> crate::Result<Self> {
        if !self.is_int()? || !other.is_int()? {
            return Err(crate::Error::NonIntegralValue);
        }
        self.gcd(other)
    }

    pub fn checked_exgcd(self, other: Self) -> crate::Result<(Self, Self, Self)> {
        if !self.is_int()? || !other.is_int()? {
            return Err(crate::Error::NonIntegralValue);
        }
        let mut x = std::ptr::null_mut();
        let mut y = std::ptr::null_mut();
        let this = ManuallyDrop::new(self);
        let other = ManuallyDrop::new(other);
        let handle = unsafe {
            barvinok_sys::isl_val_gcdext(
                this.handle.as_ptr(),
                other.handle.as_ptr(),
                &mut x,
                &mut y,
            )
        };
        let handle = nonnull_or_alloc_error(handle);
        let x = nonnull_or_alloc_error(x);
        let y = nonnull_or_alloc_error(y);
        Ok((
            Value {
                handle,
                marker: std::marker::PhantomData,
            },
            Value {
                handle: x,
                marker: std::marker::PhantomData,
            },
            Value {
                handle: y,
                marker: std::marker::PhantomData,
            },
        ))
    }
}

impl From<Value<'_>> for f64 {
    fn from(value: Value<'_>) -> Self {
        value.to_f64()
    }
}

macro_rules! impl_cmp_method {
    ($method:ident) => {
        fn $method(&self, other: &Self) -> bool {
            Value::$method(self, other).unwrap()
        }
    };
}

#[allow(clippy::partialeq_ne_impl)]
impl PartialEq for Value<'_> {
    impl_cmp_method!(eq);
    impl_cmp_method!(ne);
}

impl PartialOrd for Value<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if Value::lt(self, other).ok()? {
            Some(std::cmp::Ordering::Less)
        } else if Value::eq(self, other).ok()? {
            Some(std::cmp::Ordering::Equal)
        } else if Value::gt(self, other).ok()? {
            Some(std::cmp::Ordering::Greater)
        } else {
            None
        }
    }

    impl_cmp_method!(ge);
    impl_cmp_method!(le);
    impl_cmp_method!(gt);
    impl_cmp_method!(lt);
}

impl<'a> std::ops::Neg for Value<'a> {
    type Output = Value<'a>;

    fn neg(self) -> Self::Output {
        Value::neg(self).unwrap()
    }
}

macro_rules! impl_bin_op {
    ($trait:ident, $method:ident) => {
        impl<'a> std::ops::$trait for Value<'a> {
            type Output = Value<'a>;

            fn $method(self, other: Self) -> Self::Output {
                Value::$method(self, other).unwrap()
            }
        }
    };
}

impl_bin_op!(Add, add);
impl_bin_op!(Sub, sub);
impl_bin_op!(Mul, mul);
impl_bin_op!(Div, div);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_value() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val = Value::int_from_si(ctx, 42).unwrap();
            assert_eq!(val.numerator(), 42);
            assert_eq!(val.denominator(), 1);
            assert_eq!(val.to_f64(), 42.0);
        });
    }

    #[test]
    fn test_value_clone() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val = Value::int_from_si(ctx, 42).unwrap();
            let val_clone = val.clone();
            assert_eq!(val_clone.numerator(), 42);
            assert_eq!(val_clone.denominator(), 1);
            assert_eq!(val_clone.to_f64(), 42.0);
        });
    }

    #[test]
    fn test_value_chunks() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val = Value::new_chunks(ctx, &[0, 2, 2]);
            assert!((val.to_f64() - 2.0f64.powi(33) - 2.0f64.powi(65)).abs() < f64::EPSILON);
        });
    }

    #[test]
    fn test_value_abs_eq() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val1 = Value::int_from_si(ctx, 42).unwrap();
            let val2 = Value::int_from_si(ctx, -42).unwrap();
            assert!(val1.abs_eq(&val2).unwrap());
        });
    }

    #[test]
    fn test_value_cmp() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val1 = Value::int_from_si(ctx, 42).unwrap();
            let val2 = Value::int_from_si(ctx, 43).unwrap();
            assert!(val1 < val2);
            assert!(val1 <= val2);
            assert!(val2 > val1);
            assert!(val2 >= val1);
            assert!(val1 != val2);
            assert!(val1 == val1);
            assert!(val2 == val2);
        });
    }

    #[test]
    fn test_value_special() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val_zero = Value::zero(ctx).unwrap();
            let val_one = Value::one(ctx).unwrap();
            let val_negone = Value::negone(ctx).unwrap();
            let val_nan = Value::nan(ctx).unwrap();
            let val_infty = Value::infty(ctx).unwrap();
            let val_neg_infty = Value::neginfty(ctx).unwrap();

            assert!(val_zero.is_zero().unwrap());
            assert!(val_one.is_one().unwrap());
            assert!(val_negone.is_negone().unwrap());
            assert!(val_nan.is_nan().unwrap());
            assert!(val_infty.is_infty().unwrap());
            assert!(val_neg_infty.is_neginfty().unwrap());

            assert!(!val_zero.is_one().unwrap());
            assert!(!val_one.is_zero().unwrap());
            assert!(!val_zero.is_nan().unwrap());
            assert!(!val_one.is_nan().unwrap());
            assert!(!val_zero.is_infty().unwrap());
            assert!(!val_one.is_infty().unwrap());
            assert!(!val_zero.is_neginfty().unwrap());
            assert!(!val_one.is_neginfty().unwrap());
            assert!(!val_nan.is_zero().unwrap());
        });
    }

    #[test]
    fn test_value_divisible_by() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val1 = Value::int_from_si(ctx, 42).unwrap();
            let val2 = Value::int_from_si(ctx, 7).unwrap();
            assert!(val1.is_divisible_by(&val2).unwrap());
            assert!(!val2.is_divisible_by(&val1).unwrap());
        });
    }

    #[test]
    fn test_value_cmp_si() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val = Value::int_from_si(ctx, 42).unwrap();
            assert_eq!(val.cmp_si(42), Some(std::cmp::Ordering::Equal));
            assert_eq!(val.cmp_si(43), Some(std::cmp::Ordering::Less));
            assert_eq!(val.cmp_si(41), Some(std::cmp::Ordering::Greater));
            assert_eq!(val.cmp_si(0), Some(std::cmp::Ordering::Greater));
        });
    }

    #[test]
    fn test_unary_methods() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val = Value::int_from_si(ctx, 42).unwrap();
            assert_eq!(val.clone().abs().unwrap().to_f64(), 42.0);
            assert_eq!(val.clone().floor().unwrap().to_f64(), 42.0);
            assert_eq!(val.clone().ceil().unwrap().to_f64(), 42.0);
            assert_eq!(val.clone().trunc().unwrap().to_f64(), 42.0);
            assert_eq!(val.clone().inv().unwrap().to_f64(), 1.0 / 42.0);
            assert_eq!(val.clone().neg().unwrap().to_f64(), -42.0);
        });
    }

    #[test]
    fn test_binary_methods() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val1 = Value::int_from_si(ctx, 42).unwrap();
            let val2 = Value::int_from_si(ctx, 7).unwrap();
            assert_eq!((val1.clone() + val2.clone()).to_f64(), 49.0);
            assert_eq!((val1.clone() - val2.clone()).to_f64(), 35.0);
            assert_eq!((val1.clone() * val2.clone()).to_f64(), 294.0);
            assert_eq!((val1.clone() / val2.clone()).to_f64(), 6.0);
        });
    }

    #[test]
    fn test_binary_methods_ui() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val1 = Value::int_from_si(ctx, 42).unwrap();
            assert_eq!(val1.clone().add_ui(7).unwrap().to_f64(), 49.0);
            assert_eq!(val1.clone().sub_ui(7).unwrap().to_f64(), 35.0);
            assert_eq!(val1.clone().mul_ui(7).unwrap().to_f64(), 294.0);
            assert_eq!(val1.clone().div_ui(7).unwrap().to_f64(), 6.0);
        });
    }

    #[test]
    fn test_val_exp2() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val = Value::nan(ctx).unwrap();
            assert!(val.checked_exp2().is_err());
            let val = Value::int_from_si(ctx, 42).unwrap();
            let val = val.checked_exp2().unwrap();
            assert_eq!(val.to_f64(), 2.0f64.powi(42));
        });
    }

    #[test]
    fn test_division_like_int_operations() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val1 = Value::int_from_si(ctx, 42).unwrap();
            let val2 = Value::int_from_si(ctx, 7).unwrap();
            assert_eq!(
                val1.clone().checked_rem(val2.clone()).unwrap().to_f64(),
                0.0
            );
            assert_eq!(
                val1.clone().checked_gcd(val2.clone()).unwrap().to_f64(),
                7.0
            );
            let (gcd, x, y) = val1.checked_exgcd(val2.clone()).unwrap();
            assert_eq!(gcd.to_f64(), 7.0);
            assert_eq!(x.to_f64(), 0.0);
            assert_eq!(y.to_f64(), 1.0);
        });
    }

    #[test]
    fn test_create_val_from_ctx_ref() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val = Value::int_from_si(ctx, 42).unwrap();
            let ctx_ref = val.context_ref();
            let val2 = Value::int_from_si(ctx_ref, 42).unwrap();
            assert_eq!(val2.numerator(), 42);
            assert_eq!(val2.denominator(), 1);
            let added = val + val2;
            assert_eq!(added.numerator(), 84);
        });
    }

    #[test]
    fn test_print_inv() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val = Value::int_from_si(ctx, 42).unwrap();
            println!("val: {:?}", val);
            let val_inv = val.clone().inv().unwrap();
            println!("val_inv: {:?}", val_inv);
        });
    }

    #[test]
    fn test_new_from_string() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val = Value::from_str(ctx, "42").unwrap();
            assert_eq!(val.numerator(), 42);
            assert_eq!(val.denominator(), 1);
            assert_eq!(val.to_f64(), 42.0);

            let val = Value::from_str(ctx, "nan");
            assert!(val.unwrap().is_nan().unwrap());

            let val = Value::from_str(ctx, "infty");
            assert!(val.unwrap().is_infty().unwrap());

            let val = Value::from_str(ctx, "5/12").unwrap();
            assert_eq!(val.numerator(), 5);
            assert_eq!(val.denominator(), 12);
        });
    }

    #[test]
    fn test_dump_empty_list() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let val_list = crate::list::ValueList::new(ctx, 9);
            println!("val_list: {:?}", val_list);
        });
    }

    #[test]
    fn test_add_to_list() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let mut val_list = crate::list::ValueList::new(ctx, 9);
            let val1 = Value::int_from_si(ctx, 42).unwrap();
            let val2 = Value::int_from_si(ctx, 7).unwrap();
            val_list.push(val1);
            val_list.push(val2);
            println!("val_list: {:?}", val_list);
        });
    }
}
