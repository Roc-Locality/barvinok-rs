use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{
    DimType, impl_isl_handle,
    local_space::LocalSpace,
    nonnull_or_alloc_error,
    space::Space,
    stat::{isl_bool_to_optional_bool, isl_size_to_optional_u32},
    value::Value,
};

impl_isl_handle!(Constraint, constraint);

impl<'a> Constraint<'a> {
    pub fn new_equality(local_space: LocalSpace<'a>) -> Self {
        let local_space = ManuallyDrop::new(local_space);
        let handle =
            unsafe { barvinok_sys::isl_constraint_alloc_equality(local_space.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn new_inequality(local_space: LocalSpace<'a>) -> Self {
        let local_space = ManuallyDrop::new(local_space);
        let handle =
            unsafe { barvinok_sys::isl_constraint_alloc_inequality(local_space.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn get_space(&self) -> Space<'a> {
        let handle = unsafe { barvinok_sys::isl_constraint_get_space(self.handle.as_ptr()) };
        let handle = unsafe { NonNull::new_unchecked(handle) };
        Space {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn get_local_space(&self) -> LocalSpace<'a> {
        let handle = unsafe { barvinok_sys::isl_constraint_get_local_space(self.handle.as_ptr()) };
        let handle = unsafe { NonNull::new_unchecked(handle) };
        LocalSpace {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn dim(&self, dim_type: DimType) -> Option<u32> {
        let dim = unsafe {
            barvinok_sys::isl_constraint_dim(
                self.handle.as_ptr(),
                dim_type as barvinok_sys::isl_dim_type,
            )
        };
        isl_size_to_optional_u32(dim)
    }
    pub fn involves_dims(&self, dim_type: DimType, from: u32, num: u32) -> Option<bool> {
        let result = unsafe {
            barvinok_sys::isl_constraint_involves_dims(
                self.handle.as_ptr(),
                dim_type as barvinok_sys::isl_dim_type,
                from,
                num,
            )
        };
        isl_bool_to_optional_bool(result)
    }
    pub fn get_dim_name(&self, dim_type: DimType, pos: u32) -> Result<&str, crate::Error> {
        let name = unsafe {
            barvinok_sys::isl_constraint_get_dim_name(
                self.handle.as_ptr(),
                dim_type as barvinok_sys::isl_dim_type,
                pos,
            )
        };
        if name.is_null() {
            Err(self.context_ref().as_ref().last_error_or_unknown().into())
        } else {
            let c_str = unsafe { std::ffi::CStr::from_ptr(name) };
            let slice = c_str.to_str()?;
            Ok(slice)
        }
    }
    pub fn get_constant(&self) -> Value<'a> {
        let handle = unsafe { barvinok_sys::isl_constraint_get_constant_val(self.handle.as_ptr()) };
        let handle = unsafe { NonNull::new_unchecked(handle) };
        Value {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn get_coefficient(&self, dim_type: DimType, pos: u32) -> Option<Value<'a>> {
        let handle = unsafe {
            barvinok_sys::isl_constraint_get_coefficient_val(
                self.handle.as_ptr(),
                dim_type as barvinok_sys::isl_dim_type,
                pos as i32,
            )
        };
        NonNull::new(handle).map(|handle| Value {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn set_constant_si(self, si: i32) -> Result<Self, crate::Error> {
        let ctx = self.context_ref();
        let this = ManuallyDrop::new(self);
        let handle =
            unsafe { barvinok_sys::isl_constraint_set_constant_si(this.handle.as_ptr(), si) };
        NonNull::new(handle)
            .map(|handle| Self {
                handle,
                marker: std::marker::PhantomData,
            })
            .ok_or_else(|| ctx.as_ref().last_error_or_unknown().into())
    }
    pub fn set_constant_val(self, value: Value<'a>) -> Result<Self, crate::Error> {
        let ctx = self.context_ref();
        let this = ManuallyDrop::new(self);
        let value = ManuallyDrop::new(value);
        let handle = unsafe {
            barvinok_sys::isl_constraint_set_constant_val(
                this.handle.as_ptr(),
                value.handle.as_ptr(),
            )
        };
        NonNull::new(handle)
            .map(|handle| Self {
                handle,
                marker: std::marker::PhantomData,
            })
            .ok_or_else(|| ctx.as_ref().last_error_or_unknown().into())
    }
    pub fn set_coefficient_si(
        self,
        dim_type: DimType,
        pos: u32,
        si: i32,
    ) -> Result<Self, crate::Error> {
        let ctx = self.context_ref();
        let this = ManuallyDrop::new(self);
        let handle = unsafe {
            barvinok_sys::isl_constraint_set_coefficient_si(
                this.handle.as_ptr(),
                dim_type as barvinok_sys::isl_dim_type,
                pos as i32,
                si,
            )
        };
        NonNull::new(handle)
            .map(|handle| Self {
                handle,
                marker: std::marker::PhantomData,
            })
            .ok_or_else(|| ctx.as_ref().last_error_or_unknown().into())
    }
    pub fn set_coefficient_val(
        self,
        dim_type: DimType,
        pos: u32,
        value: Value<'a>,
    ) -> Result<Self, crate::Error> {
        let ctx = self.context_ref();
        let this = ManuallyDrop::new(self);
        let value = ManuallyDrop::new(value);
        let handle = unsafe {
            barvinok_sys::isl_constraint_set_coefficient_val(
                this.handle.as_ptr(),
                dim_type as barvinok_sys::isl_dim_type,
                pos as i32,
                value.handle.as_ptr(),
            )
        };
        NonNull::new(handle)
            .map(|handle| Self {
                handle,
                marker: std::marker::PhantomData,
            })
            .ok_or_else(|| ctx.as_ref().last_error_or_unknown().into())
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;

    use super::*;
    use crate::local_space::LocalSpace;
    use crate::space::Space;

    #[test]
    fn test_new_equality() {
        let context = Context::new();
        let space = Space::new(&context, 1, 2, 2);
        let local_space = LocalSpace::from(space);
        let constraint = Constraint::new_equality(local_space);
        println!("Constraint: {:?}", constraint);
    }

    #[test]
    fn test_new_inequality() {
        let context = Context::new();
        let space = Space::new(&context, 1, 2, 2);
        let local_space = LocalSpace::from(space);
        let constraint = Constraint::new_inequality(local_space);
        println!("Constraint: {:?}", constraint);
    }

    #[test]
    fn test_inequality_constant_and_coeff() {
        let context = Context::new();
        let space = Space::new(&context, 1, 2, 2);
        let local_space = LocalSpace::from(space);
        let constraint = Constraint::new_inequality(local_space);
        let constant_val = constraint.get_constant();
        println!("Constant Value: {:?}", constant_val);
        let coeff_val = constraint.get_coefficient(DimType::Param, 0);
        println!("Coefficient Value: {:?}", coeff_val);
    }

    #[test]
    fn test_set_constant_and_coeff() {
        let context = Context::new();
        let space = Space::new(&context, 1, 2, 2);
        let local_space = LocalSpace::from(space);
        let mut constraint = Constraint::new_inequality(local_space);
        constraint = constraint.set_constant_si(5).unwrap();
        constraint = constraint.set_coefficient_si(DimType::Param, 0, 3).unwrap();
        println!("Updated Constraint: {:?}", constraint);
    }
}
