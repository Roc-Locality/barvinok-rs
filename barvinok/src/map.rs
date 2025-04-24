use std::ptr::NonNull;

use crate::{impl_isl_print, nonnull_or_alloc_error};

#[repr(transparent)]
pub struct BasicMap<'a> {
    pub(crate) handle: NonNull<barvinok_sys::isl_basic_map>,
    pub(crate) marker: std::marker::PhantomData<*mut &'a ()>,
}

#[repr(transparent)]
pub struct Map<'a> {
    pub(crate) handle: NonNull<barvinok_sys::isl_map>,
    pub(crate) marker: std::marker::PhantomData<*mut &'a ()>,
}

impl<'a> Map<'a> {
    pub fn context_ref(&self) -> crate::ContextRef<'a> {
        let handle = unsafe { barvinok_sys::isl_map_get_ctx(self.handle.as_ptr()) };
        let handle = unsafe { NonNull::new_unchecked(handle) };
        crate::ContextRef(handle, std::marker::PhantomData)
    }
}

impl Drop for Map<'_> {
    fn drop(&mut self) {
        unsafe { barvinok_sys::isl_map_free(self.handle.as_ptr()) };
    }
}
impl Clone for Map<'_> {
    fn clone(&self) -> Self {
        let handle = unsafe { barvinok_sys::isl_map_copy(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a> BasicMap<'a> {
    pub fn context_ref(&self) -> crate::ContextRef<'a> {
        let handle = unsafe { barvinok_sys::isl_basic_map_get_ctx(self.handle.as_ptr()) };
        let handle = unsafe { NonNull::new_unchecked(handle) };
        crate::ContextRef(handle, std::marker::PhantomData)
    }
}

impl Drop for BasicMap<'_> {
    fn drop(&mut self) {
        unsafe { barvinok_sys::isl_basic_map_free(self.handle.as_ptr()) };
    }
}

impl Clone for BasicMap<'_> {
    fn clone(&self) -> Self {
        let handle = unsafe { barvinok_sys::isl_basic_map_copy(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl_isl_print!(Map, isl_map, isl_printer_print_map);
impl_isl_print!(BasicMap, isl_basic_map, isl_printer_print_basic_map);
