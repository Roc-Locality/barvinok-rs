use std::ffi::CStr;
use std::mem::ManuallyDrop;

use barvinok_sys::isl_dim_type;

use crate::stat::isl_size_to_optional_u32;
use crate::{
    Context, ident::Ident, nonnull_or_alloc_error, stat::isl_bool_to_optional_bool,
};
use crate::{impl_isl_handle, DimType};

impl_isl_handle!(Space, space);

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

macro_rules! space_flag {
    ($name:ident, $sys_fn:ident) => {
        pub fn $name(&self) -> Option<bool> {
            let flag = unsafe { barvinok_sys::$sys_fn(self.handle.as_ptr()) };
            isl_bool_to_optional_bool(flag)
        }
    };
}

impl<'a> Space<'a> {
    space_constructor!(new,        isl_space_alloc,    num_params: u32, num_inputs: u32, num_outputs: u32);
    space_constructor!(new_set,    isl_space_set_alloc,    num_params: u32, num_dims: u32);
    space_constructor!(new_params, isl_space_params_alloc, num_params: u32);
    space_constructor!(new_unit, isl_space_unit);
    space_flag!(is_params, isl_space_is_params);
    space_flag!(is_set, isl_space_is_set);
    space_flag!(is_map, isl_space_is_map);

    pub fn add_param_id(&mut self, id: Ident<'a>) {
        let id = ManuallyDrop::new(id);
        let added = unsafe {
            barvinok_sys::isl_space_add_param_id(self.handle.as_ptr(), id.handle.as_ptr())
        };
        let added = nonnull_or_alloc_error(added);
        self.handle = added;
    }
    pub fn set_tuple_name(&mut self, dim_type: DimType, name: &str) -> Result<(), crate::Error> {
        let name = std::ffi::CString::new(name)?;
        let name = name.as_ptr();
        let handle = unsafe {
            barvinok_sys::isl_space_set_tuple_name(
                self.handle.as_ptr(),
                dim_type as isl_dim_type,
                name,
            )
        };
        self.handle = nonnull_or_alloc_error(handle);
        Ok(())
    }
    pub fn has_tuple_name(&self, dim_type: DimType) -> Option<bool> {
        let flag = unsafe {
            barvinok_sys::isl_space_has_tuple_name(self.handle.as_ptr(), dim_type as isl_dim_type)
        };
        isl_bool_to_optional_bool(flag)
    }
    pub fn get_tuple_name(&self, dim_type: DimType) -> Result<&str, crate::Error> {
        let cstr = self.get_tuple_name_cstr(dim_type);
        Ok(cstr.to_str()?)
    }
    pub fn get_tuple_name_cstr(&self, dim_type: DimType) -> &CStr {
        let cstr = unsafe {
            barvinok_sys::isl_space_get_tuple_name(self.handle.as_ptr(), dim_type as isl_dim_type)
        };
        unsafe { CStr::from_ptr(cstr) }
    }
    pub fn add_dims(&mut self, dim_type: DimType, num: u32) {
        let handle = unsafe {
            barvinok_sys::isl_space_add_dims(self.handle.as_ptr(), dim_type as isl_dim_type, num)
        };
        self.handle = nonnull_or_alloc_error(handle);
    }
    pub fn get_dim(&self, dim_type: DimType) -> Option<u32> {
        let dim =
            unsafe { barvinok_sys::isl_space_dim(self.handle.as_ptr(), dim_type as isl_dim_type) };
        isl_size_to_optional_u32(dim)
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

    #[test]
    fn test_space_add_param_id() {
        let ctx = Context::new();
        let mut space = Space::new_set(&ctx, 2, 4);
        let id = Ident::new(&ctx, "x").unwrap();
        space.add_param_id(id);
        println!("{:?}", space);
    }

    #[test]
    fn test_space_set_tuple_name() {
        let ctx = Context::new();
        let mut space = Space::new_set(&ctx, 2, 4);
        space.set_tuple_name(DimType::In, "input").unwrap();
        println!("{:?}", space);
        assert!(space.has_tuple_name(DimType::In).unwrap());
        assert_eq!(space.get_tuple_name(DimType::In).unwrap(), "input");
    }

    #[test]
    fn test_space_add_dims() {
        let ctx = Context::new();
        let mut space = Space::new_set(&ctx, 2, 4);
        assert!(space.is_set().unwrap());
        space.add_dims(DimType::In, 2);
        println!("{:?}", space);
        assert!(space.is_map().unwrap());
    }
}
