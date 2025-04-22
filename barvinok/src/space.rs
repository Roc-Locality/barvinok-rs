use std::ptr::NonNull;
use std::{ffi::CStr, fmt::Debug};

use barvinok_sys::isl_dim_type;

use crate::DimType;
use crate::{
    Context, ContextRef, ident::Ident, nonnull_or_alloc_error, printer::ISLPrint, stat::Flag,
};

#[repr(transparent)]
pub struct Space<'a> {
    pub(crate) handle: NonNull<barvinok_sys::isl_space>,
    pub(crate) marker: std::marker::PhantomData<*mut &'a ()>,
}

impl<'a> ISLPrint<'a> for Space<'a> {
    type Handle = barvinok_sys::isl_space;

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
        unsafe { barvinok_sys::isl_printer_print_space(printer, handle) }
    }
}

impl Debug for Space<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wrapper = crate::printer::FmtWrapper::new(self);
        Debug::fmt(&wrapper, f)
    }
}

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
        pub fn $name(&self) -> bool {
            let flag = unsafe { barvinok_sys::$sys_fn(self.handle.as_ptr()) };
            let flag = Flag::from_isl_bool(flag);
            matches!(flag, Flag::True)
        }
    };
}

impl<'a> Space<'a> {
    space_constructor!(new,        isl_space_alloc,    num_params: u32, num_inputs: u32, num_outputs: u32);
    space_constructor!(new_set,    isl_space_set_alloc,    num_params: u32, num_dims: u32);
    space_constructor!(new_params, isl_space_params_alloc, num_params: u32);
    space_constructor!(new_unit, isl_space_unit);

    pub fn context_ref(&self) -> ContextRef<'a> {
        ContextRef(
            unsafe {
                NonNull::new_unchecked(barvinok_sys::isl_space_get_ctx(self.handle.as_ptr()))
            },
            std::marker::PhantomData,
        )
    }

    space_flag!(is_params, isl_space_is_params);
    space_flag!(is_set, isl_space_is_set);
    space_flag!(is_map, isl_space_is_map);

    pub fn add_param_id(&mut self, id: Ident<'a>) {
        let added = unsafe {
            barvinok_sys::isl_space_add_param_id(self.handle.as_ptr(), id.handle.as_ptr())
        };
        let added = nonnull_or_alloc_error(added);
        std::mem::forget(id);
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
    pub fn has_tuple_name(&self, dim_type: DimType) -> bool {
        let flag = unsafe {
            barvinok_sys::isl_space_has_tuple_name(self.handle.as_ptr(), dim_type as isl_dim_type)
        };
        let flag = Flag::from_isl_bool(flag);
        matches!(flag, Flag::True)
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
        if dim >= 0 { Some(dim as u32) } else { None }
    }
}

impl Drop for Space<'_> {
    fn drop(&mut self) {
        unsafe { barvinok_sys::isl_space_free(self.handle.as_ptr()) };
    }
}

impl Clone for Space<'_> {
    fn clone(&self) -> Self {
        let handle = unsafe { barvinok_sys::isl_space_copy(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
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
        assert!(space.has_tuple_name(DimType::In));
        assert_eq!(space.get_tuple_name(DimType::In).unwrap(), "input");
    }

    #[test]
    fn test_space_add_dims() {
        let ctx = Context::new();
        let mut space = Space::new_set(&ctx, 2, 4);
        assert!(space.is_set());
        space.add_dims(DimType::In, 2);
        println!("{:?}", space);
        assert!(space.is_map());
    }
}
