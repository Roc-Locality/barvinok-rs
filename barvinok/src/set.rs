use std::ptr::NonNull;

use crate::{
    DimType, impl_isl_print, nonnull_or_alloc_error,
    space::Space,
    stat::{ContextResult, isl_size_to_optional_u32},
};

#[repr(transparent)]
pub struct BasicSet<'a> {
    pub(crate) handle: NonNull<barvinok_sys::isl_basic_set>,
    pub(crate) marker: std::marker::PhantomData<*mut &'a ()>,
}

#[repr(transparent)]
pub struct Set<'a> {
    pub(crate) handle: NonNull<barvinok_sys::isl_set>,
    pub(crate) marker: std::marker::PhantomData<*mut &'a ()>,
}

impl<'a> Set<'a> {
    pub fn context_ref(&self) -> crate::ContextRef<'a> {
        let handle = unsafe { barvinok_sys::isl_set_get_ctx(self.handle.as_ptr()) };
        let handle = unsafe { NonNull::new_unchecked(handle) };
        crate::ContextRef(handle, std::marker::PhantomData)
    }
}

impl Drop for Set<'_> {
    fn drop(&mut self) {
        unsafe { barvinok_sys::isl_set_free(self.handle.as_ptr()) };
    }
}
impl Clone for Set<'_> {
    fn clone(&self) -> Self {
        let handle = unsafe { barvinok_sys::isl_set_copy(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

macro_rules! basic_set_constructor {
    ($fn_name:ident, $isl_fn:ident) => {
        paste::paste! {
            pub fn [<new_ $fn_name>](space: Space<'a>) -> Result<Self, crate::Error> {
                if space.get_dim(crate::DimType::In)
                    .context_result(space.context_ref())? != 0
                {
                    return Err(crate::ISLError::Invalid.into());
                }
                let handle = unsafe { barvinok_sys::$isl_fn(space.handle.as_ptr()) };
                let handle = nonnull_or_alloc_error(handle);
                std::mem::forget(space);
                Ok(Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
            }
        }
    };
}

macro_rules! basic_set_unary {
    ($fn_name:ident) => {
        paste::paste! {
            pub fn $fn_name(self) -> Result<Self, crate::Error> {
                let handle = unsafe { barvinok_sys::[<isl_basic_set_ $fn_name>](self.handle.as_ptr()) };
                let handle = nonnull_or_alloc_error(handle);
                std::mem::forget(self);
                Ok(Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
            }
        }
    };
}

macro_rules! basic_set_binary {
    ($fn_name:ident) => {
        paste::paste! {
            pub fn $fn_name(self, other: BasicSet<'a>) -> Result<Self, crate::Error> {
                let handle = unsafe {
                    barvinok_sys::[<isl_basic_set_ $fn_name>](self.handle.as_ptr(), other.handle.as_ptr())
                };
                let handle = nonnull_or_alloc_error(handle);
                std::mem::forget(self);
                std::mem::forget(other);
                Ok(Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
            }
        }
    };
}

impl<'a> BasicSet<'a> {
    pub fn context_ref(&self) -> crate::ContextRef<'a> {
        let handle = unsafe { barvinok_sys::isl_basic_set_get_ctx(self.handle.as_ptr()) };
        let handle = unsafe { NonNull::new_unchecked(handle) };
        crate::ContextRef(handle, std::marker::PhantomData)
    }
    basic_set_constructor!(universe, isl_basic_set_universe);
    basic_set_constructor!(empty, isl_basic_set_empty);
    basic_set_constructor!(nat_universe, isl_basic_set_nat_universe);
    basic_set_constructor!(positive_orthant, isl_basic_set_positive_orthant);
    pub fn num_dims(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_n_dim(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn num_params(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_n_param(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn total_dims(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_total_dim(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn get_dims(&self, ty: DimType) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_dim(self.handle.as_ptr(), ty as u32) };
        isl_size_to_optional_u32(num)
    }
    basic_set_binary!(intersect);
    basic_set_binary!(intersect_params);
    basic_set_unary!(affine_hull);
    basic_set_unary!(sample);
    basic_set_unary!(remove_redundancies);
}

impl Drop for BasicSet<'_> {
    fn drop(&mut self) {
        unsafe { barvinok_sys::isl_basic_set_free(self.handle.as_ptr()) };
    }
}

impl Clone for BasicSet<'_> {
    fn clone(&self) -> Self {
        let handle = unsafe { barvinok_sys::isl_basic_set_copy(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl_isl_print!(Set, isl_set, isl_printer_print_set);
impl_isl_print!(BasicSet, isl_basic_set, isl_printer_print_basic_set);

#[cfg(test)]
mod test {
    use super::*;
    use crate::Context;

    #[test]
    fn test_basic_set_creation() {
        let ctx = Context::new();
        let space = Space::new(&ctx, 0, 0, 3);
        let basic_set = BasicSet::new_universe(space.clone()).unwrap();
        println!("{:?}", basic_set);
        let basic_set = BasicSet::new_empty(space.clone()).unwrap();
        println!("{:?}", basic_set);
        let basic_set = BasicSet::new_nat_universe(space.clone()).unwrap();
        println!("{:?}", basic_set);
        let basic_set = BasicSet::new_positive_orthant(space.clone()).unwrap();
        println!("{:?}", basic_set);
    }

    #[test]
    fn test_basic_set_bin_ops() {
        let ctx = Context::new();
        let space = Space::new(&ctx, 0, 0, 3);
        let basic_set1 = BasicSet::new_universe(space.clone()).unwrap();
        let basic_set2 = BasicSet::new_empty(space.clone()).unwrap();
        let basic_set3 = basic_set1.intersect(basic_set2).unwrap();
        println!("{:?}", basic_set3);
    }

    #[test]
    fn test_basic_set_unary_ops() {
        let ctx = Context::new();
        let space = Space::new(&ctx, 0, 0, 3);
        let basic_set = BasicSet::new_positive_orthant(space.clone()).unwrap();
        let basic_set = basic_set.affine_hull().unwrap();
        println!("{:?}", basic_set);
    }
}
