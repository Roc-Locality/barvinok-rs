use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{Context, impl_isl_handle, stat::isl_size_to_optional_u32, value::Value};

impl_isl_handle!([printer] Vector, vec);

impl<'a> Vector<'a> {
    pub fn new(ctx: &'a Context, dim: u32) -> Self {
        let handle = unsafe { barvinok_sys::isl_vec_alloc(ctx.0.as_ptr(), dim) };
        let handle = std::ptr::NonNull::new(handle).expect("Failed to allocate vector");
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn new_zero(ctx: &'a Context, dim: u32) -> Self {
        let handle = unsafe { barvinok_sys::isl_vec_zero(ctx.0.as_ptr(), dim) };
        let handle = std::ptr::NonNull::new(handle).expect("Failed to allocate zero vector");
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn size(&self) -> Option<u32> {
        let size = unsafe { barvinok_sys::isl_vec_size(self.handle.as_ptr()) };
        isl_size_to_optional_u32(size)
    }
    pub fn get(&self, index: u32) -> Option<Value<'a>> {
        let val =
            unsafe { barvinok_sys::isl_vec_get_element_val(self.handle.as_ptr(), index as i32) };
        NonNull::new(val).map(|v| Value {
            handle: v,
            marker: std::marker::PhantomData,
        })
    }
    pub fn set_si(self, pos: u32, val: i32) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let handle =
            unsafe { barvinok_sys::isl_vec_set_element_si(this.handle.as_ptr(), pos as i32, val) };
        NonNull::new(handle).map(|h| Self {
            handle: h,
            marker: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_vector_creation() {
        let ctx = Context::new();
        let vector = Vector::new(&ctx, 5);
        println!("{:?}", vector);
        assert_eq!(vector.size(), Some(5));
    }

    #[test]
    fn test_get_as_val() {
        let ctx = Context::new();
        let vector = Vector::new(&ctx, 5);
        let val = vector.get(10);
        assert!(val.is_none());
        let val = vector.get(0).unwrap();
        assert!(val.is_zero().unwrap());
    }

    #[test]
    fn test_set_si() {
        let ctx = Context::new();
        let mut vector = Vector::new(&ctx, 5);
        vector = vector.set_si(0, 10).unwrap();
        let val = vector.get(0).unwrap();
        assert_eq!(val.to_f64(), 10f64);
        assert!(vector.set_si(99, 20).is_none());
    }
}
