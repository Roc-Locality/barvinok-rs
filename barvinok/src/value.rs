use std::{fmt::Debug, ptr::NonNull};

use num_traits::PrimInt;

use crate::{
    Context, ContextRef, nonnull_or_alloc_error,
    printer::ISLPrint,
    stat::{ContextResult, isl_bool_to_optional_bool},
};

#[repr(transparent)]
pub struct Value<'a> {
    pub(crate) handle: NonNull<barvinok_sys::isl_val>,
    pub(crate) marker: std::marker::PhantomData<*mut &'a ()>,
}

impl<'a> ISLPrint<'a> for Value<'a> {
    type Handle = barvinok_sys::isl_val;

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
        unsafe { barvinok_sys::isl_printer_print_val(printer, handle) }
    }
}

impl Debug for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wrapper = crate::printer::FmtWrapper::new(self);
        Debug::fmt(&wrapper, f)
    }
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

macro_rules! impl_special_val_check {
    ($method:ident, $isl_fn:ident) => {
        pub fn $method(&self) -> Option<bool> {
            let flag = unsafe { barvinok_sys::$isl_fn(self.handle.as_ptr()) };
            isl_bool_to_optional_bool(flag)
        }
    };
}

macro_rules! impl_unary_method {
    ($method:ident, $isl_fn:ident) => {
        pub fn $method(self) -> Self {
            let handle = unsafe { barvinok_sys::$isl_fn(self.handle.as_ptr()) };
            let handle = nonnull_or_alloc_error(handle);
            std::mem::forget(self);
            Self {
                handle,
                marker: std::marker::PhantomData,
            }
        }
    };
}

macro_rules! impl_binary_method {
    ($method:ident, $isl_fn:ident) => {
        pub fn $method(self, other: Self) -> Self {
            let handle =
                unsafe { barvinok_sys::$isl_fn(self.handle.as_ptr(), other.handle.as_ptr()) };
            std::mem::forget(self);
            std::mem::forget(other);
            let handle = nonnull_or_alloc_error(handle);
            Self {
                handle,
                marker: std::marker::PhantomData,
            }
        }
    };
}

macro_rules! impl_binary_method_ui {
    ($method:ident, $isl_fn:ident) => {
        pub fn $method(self, val: u64) -> Self {
            let handle = unsafe { barvinok_sys::$isl_fn(self.handle.as_ptr(), val) };
            let handle = nonnull_or_alloc_error(handle);
            std::mem::forget(self);
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

    pub fn context_ref(&self) -> ContextRef<'a> {
        let ctx = unsafe { barvinok_sys::isl_val_get_ctx(self.handle.as_ptr()) };
        let ctx = unsafe { NonNull::new_unchecked(ctx) };
        ContextRef(ctx, std::marker::PhantomData)
    }

    pub fn dump(&self) {
        unsafe { barvinok_sys::isl_val_dump(self.handle.as_ptr()) }
    }

    pub fn numerator(&self) -> i64 {
        unsafe { barvinok_sys::isl_val_get_num_si(self.handle.as_ptr()) }
    }

    pub fn denominator(&self) -> i64 {
        unsafe { barvinok_sys::isl_val_get_den_si(self.handle.as_ptr()) }
    }

    pub fn new_from_string(ctx: &'a Context, value: &str) -> crate::Result<Self> {
        let cstr = std::ffi::CString::new(value).map_err(|_| crate::Error::ParseError)?;
        let handle = unsafe { barvinok_sys::isl_val_read_from_str(ctx.0.as_ptr(), cstr.as_ptr()) };
        if handle.is_null() {
            return Err(crate::Error::ParseError);
        }
        let handle = nonnull_or_alloc_error(handle);
        Ok(Self {
            handle,
            marker: std::marker::PhantomData,
        })
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

    pub fn abs_eq(&self, other: &Self) -> Option<bool> {
        let flag =
            unsafe { barvinok_sys::isl_val_abs_eq(self.handle.as_ptr(), other.handle.as_ptr()) };
        isl_bool_to_optional_bool(flag)
    }
    pub fn divisible_by(&self, other: &Self) -> Option<bool> {
        let flag = unsafe {
            barvinok_sys::isl_val_is_divisible_by(self.handle.as_ptr(), other.handle.as_ptr())
        };
        isl_bool_to_optional_bool(flag)
    }

    impl_special_val_check!(is_zero, isl_val_is_zero);
    impl_special_val_check!(is_one, isl_val_is_one);
    impl_special_val_check!(is_negone, isl_val_is_negone);
    impl_special_val_check!(is_nan, isl_val_is_nan);
    impl_special_val_check!(is_infty, isl_val_is_infty);
    impl_special_val_check!(is_neg_infty, isl_val_is_neginfty);
    impl_special_val_check!(is_nonneg, isl_val_is_nonneg);
    impl_special_val_check!(is_nonpos, isl_val_is_nonpos);
    impl_special_val_check!(is_int, isl_val_is_int);
    impl_special_val_check!(is_rat, isl_val_is_rat);

    pub fn gt_si(&self, value: i64) -> Option<bool> {
        let flag = unsafe { barvinok_sys::isl_val_gt_si(self.handle.as_ptr(), value) };
        isl_bool_to_optional_bool(flag)
    }
    pub fn eq_si(&self, value: i64) -> Option<bool> {
        let flag = unsafe { barvinok_sys::isl_val_eq_si(self.handle.as_ptr(), value) };
        isl_bool_to_optional_bool(flag)
    }
    pub fn cmp_si(&self, value: i64) -> Option<std::cmp::Ordering> {
        if self.is_nan()? {
            return None;
        }
        let int_val = unsafe { barvinok_sys::isl_val_cmp_si(self.handle.as_ptr(), value) };
        match int_val.cmp(&0) {
            std::cmp::Ordering::Greater => Some(std::cmp::Ordering::Greater),
            std::cmp::Ordering::Less => Some(std::cmp::Ordering::Less),
            std::cmp::Ordering::Equal => Some(std::cmp::Ordering::Equal),
        }
    }
    impl_unary_method!(abs, isl_val_abs);
    impl_unary_method!(floor, isl_val_floor);
    impl_unary_method!(ceil, isl_val_ceil);
    impl_unary_method!(trunc, isl_val_trunc);
    impl_unary_method!(inv, isl_val_inv);
    impl_binary_method!(min, isl_val_min);
    impl_binary_method!(max, isl_val_max);

    impl_binary_method_ui!(add_ui, isl_val_add_ui);
    impl_binary_method_ui!(sub_ui, isl_val_sub_ui);
    impl_binary_method_ui!(mul_ui, isl_val_mul_ui);
    impl_binary_method_ui!(div_ui, isl_val_div_ui);

    pub fn checked_exp2(self) -> crate::Result<Self> {
        if !self.is_int().context_result(self.context_ref())? {
            return Err(crate::Error::NonIntegralValue);
        }
        let handle = unsafe { barvinok_sys::isl_val_pow2(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        std::mem::forget(self);
        Ok(Self {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn checked_rem(self, other: Self) -> crate::Result<Self> {
        if !self.is_int().context_result(self.context_ref())?
            || !other.is_int().context_result(self.context_ref())?
        {
            return Err(crate::Error::NonIntegralValue);
        }
        let handle =
            unsafe { barvinok_sys::isl_val_mod(self.handle.as_ptr(), other.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        std::mem::forget(self);
        std::mem::forget(other);
        Ok(Self {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn checked_gcd(self, other: Self) -> crate::Result<Self> {
        if !self.is_int().context_result(self.context_ref())?
            || !other.is_int().context_result(self.context_ref())?
        {
            return Err(crate::Error::NonIntegralValue);
        }
        let handle =
            unsafe { barvinok_sys::isl_val_gcd(self.handle.as_ptr(), other.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        std::mem::forget(self);
        std::mem::forget(other);
        Ok(Self {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn checked_exgcd(self, other: Self) -> crate::Result<(Self, Self, Self)> {
        if !self.is_int().context_result(self.context_ref())?
            || !other.is_int().context_result(self.context_ref())?
        {
            return Err(crate::Error::NonIntegralValue);
        }
        let mut x = std::ptr::null_mut();
        let mut y = std::ptr::null_mut();
        let handle = unsafe {
            barvinok_sys::isl_val_gcdext(
                self.handle.as_ptr(),
                other.handle.as_ptr(),
                &mut x,
                &mut y,
            )
        };
        let handle = nonnull_or_alloc_error(handle);
        let x = nonnull_or_alloc_error(x);
        let y = nonnull_or_alloc_error(y);
        std::mem::forget(self);
        std::mem::forget(other);
        let gcd = Value {
            handle,
            marker: std::marker::PhantomData,
        };
        let x = Value {
            handle: x,
            marker: std::marker::PhantomData,
        };
        let y = Value {
            handle: y,
            marker: std::marker::PhantomData,
        };
        Ok((gcd, x, y))
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
            isl_bool_to_optional_bool(flag)
                .context_result(self.context_ref())
                .unwrap()
        }
    };
}

#[allow(clippy::partialeq_ne_impl)]
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

impl<'a> std::ops::Neg for Value<'a> {
    type Output = Value<'a>;
    fn neg(self) -> Self::Output {
        let handle = unsafe { barvinok_sys::isl_val_neg(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        std::mem::forget(self);
        Value {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

macro_rules! impl_bin_op {
    ($trait:ident, $method:ident, $isl_fn:ident) => {
        impl<'a> std::ops::$trait for Value<'a> {
            type Output = Value<'a>;
            fn $method(self, other: Self) -> Self::Output {
                let handle =
                    unsafe { barvinok_sys::$isl_fn(self.handle.as_ptr(), other.handle.as_ptr()) };
                let handle = nonnull_or_alloc_error(handle);
                std::mem::forget(self);
                std::mem::forget(other);
                Value {
                    handle,
                    marker: std::marker::PhantomData,
                }
            }
        }
    };
}

impl_bin_op!(Add, add, isl_val_add);
impl_bin_op!(Sub, sub, isl_val_sub);
impl_bin_op!(Mul, mul, isl_val_mul);
impl_bin_op!(Div, div, isl_val_div);

#[cfg(test)]
mod tests {
    use std::ops::Neg;

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
        assert!(val1.abs_eq(&val2).unwrap());
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

    #[test]
    fn test_value_special() {
        let ctx = Context::new();
        let val_zero = Value::new_zero(&ctx);
        let val_one = Value::new_one(&ctx);
        let val_negone = Value::new_negone(&ctx);
        let val_nan = Value::new_nan(&ctx);
        let val_infty = Value::new_infty(&ctx);
        let val_neg_infty = Value::new_neg_infty(&ctx);

        assert!(val_zero.is_zero().unwrap());
        assert!(val_one.is_one().unwrap());
        assert!(val_negone.is_negone().unwrap());
        assert!(val_nan.is_nan().unwrap());
        assert!(val_infty.is_infty().unwrap());
        assert!(val_neg_infty.is_neg_infty().unwrap());

        // some random cross checkings
        assert!(!val_zero.is_one().unwrap());
        assert!(!val_one.is_zero().unwrap());
        assert!(!val_zero.is_nan().unwrap());
        assert!(!val_one.is_nan().unwrap());
        assert!(!val_zero.is_infty().unwrap());
        assert!(!val_one.is_infty().unwrap());
        assert!(!val_zero.is_neg_infty().unwrap());
        assert!(!val_one.is_neg_infty().unwrap());
        assert!(!val_nan.is_zero().unwrap());
    }

    #[test]
    fn test_value_divisible_by() {
        let ctx = Context::new();
        let val1 = Value::new_si(&ctx, 42);
        let val2 = Value::new_si(&ctx, 7);
        assert!(val1.divisible_by(&val2).unwrap());
        assert!(!val2.divisible_by(&val1).unwrap());
    }

    #[test]
    fn test_value_cmp_si() {
        let ctx = Context::new();
        let val = Value::new_si(&ctx, 42);
        assert_eq!(val.cmp_si(42), Some(std::cmp::Ordering::Equal));
        assert_eq!(val.cmp_si(43), Some(std::cmp::Ordering::Less));
        assert_eq!(val.cmp_si(41), Some(std::cmp::Ordering::Greater));
        assert_eq!(val.cmp_si(0), Some(std::cmp::Ordering::Greater));
    }

    #[test]
    fn test_unary_methods() {
        let ctx = Context::new();
        let val = Value::new_si(&ctx, 42);
        assert_eq!(val.clone().abs().to_f64(), 42.0);
        assert_eq!(val.clone().floor().to_f64(), 42.0);
        assert_eq!(val.clone().ceil().to_f64(), 42.0);
        assert_eq!(val.clone().trunc().to_f64(), 42.0);
        assert_eq!(val.clone().inv().to_f64(), 1.0 / 42.0);
        assert_eq!(val.clone().neg().to_f64(), -42.0);
    }

    #[test]
    fn test_binary_methods() {
        let ctx = Context::new();
        let val1 = Value::new_si(&ctx, 42);
        let val2 = Value::new_si(&ctx, 7);
        assert_eq!((val1.clone() + val2.clone()).to_f64(), 49.0);
        assert_eq!((val1.clone() - val2.clone()).to_f64(), 35.0);
        assert_eq!((val1.clone() * val2.clone()).to_f64(), 294.0);
        assert_eq!((val1.clone() / val2.clone()).to_f64(), 6.0);
    }

    #[test]
    fn test_binary_methods_ui() {
        let ctx = Context::new();
        let val = Value::new_si(&ctx, 42);
        assert_eq!(val.clone().add_ui(7).to_f64(), 49.0);
        assert_eq!(val.clone().sub_ui(7).to_f64(), 35.0);
        assert_eq!(val.clone().mul_ui(7).to_f64(), 294.0);
        assert_eq!(val.clone().div_ui(7).to_f64(), 6.0);
    }

    #[test]
    fn test_val_exp2() {
        let ctx = Context::new();
        let val = Value::new_nan(&ctx);
        assert!(val.checked_exp2().is_err());
        let val = Value::new_si(&ctx, 42);
        let val = val.checked_exp2().unwrap();
        assert_eq!(val.to_f64(), 2.0f64.powi(42));
    }

    #[test]
    fn test_division_like_int_operations() {
        let ctx = Context::new();
        let val1 = Value::new_si(&ctx, 42);
        let val2 = Value::new_si(&ctx, 7);
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
    }

    #[test]
    fn test_create_val_from_ctx_ref() {
        let ctx = Context::new();
        let val = Value::new_si(&ctx, 42);
        let ctx_ref = val.context_ref();
        let val2 = Value::new_si(ctx_ref.as_ref(), 42);
        assert_eq!(val2.numerator(), 42);
        assert_eq!(val2.denominator(), 1);
    }

    #[test]
    fn test_print_inv() {
        let ctx = Context::new();
        let val = Value::new_si(&ctx, 42);
        println!("val: {:?}", val);
        let val_inv = val.clone().inv();
        println!("val_inv: {:?}", val_inv);
    }

    #[test]
    fn test_new_from_string() {
        let ctx = Context::new();
        let val = Value::new_from_string(&ctx, "42").unwrap();
        assert_eq!(val.numerator(), 42);
        assert_eq!(val.denominator(), 1);
        assert_eq!(val.to_f64(), 42.0);

        let val = Value::new_from_string(&ctx, "nan");
        assert!(val.unwrap().is_nan().unwrap());

        let val = Value::new_from_string(&ctx, "infty");
        assert!(val.unwrap().is_infty().unwrap());

        let val = Value::new_from_string(&ctx, "5/12").unwrap();
        assert_eq!(val.numerator(), 5);
        assert_eq!(val.denominator(), 12);
    }

    #[test]
    fn test_dump_empty_list() {
        type ValueList<'a> = crate::list::List<'a, Value<'a>>;
        let ctx = Context::new();
        let val_list = ValueList::new(&ctx, 9);
        println!("val_list: {:?}", val_list);
    }

    #[test]
    fn test_add_to_list() {
        type ValueList<'a> = crate::list::List<'a, Value<'a>>;
        let ctx = Context::new();
        let mut val_list = ValueList::new(&ctx, 9);
        let val1 = Value::new_si(&ctx, 42);
        let val2 = Value::new_si(&ctx, 7);
        val_list.push(val1);
        val_list.push(val2);
        println!("val_list: {:?}", val_list);
    }
}
