use std::ptr::NonNull;

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
        let handle = unsafe { NonNull::new_unchecked(handle) };
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a> BasicSet<'a> {
    pub fn context_ref(&self) -> crate::ContextRef<'a> {
        let handle = unsafe { barvinok_sys::isl_basic_set_get_ctx(self.handle.as_ptr()) };
        let handle = unsafe { NonNull::new_unchecked(handle) };
        crate::ContextRef(handle, std::marker::PhantomData)
    }
}

impl Drop for BasicSet<'_> {
    fn drop(&mut self) {
        unsafe { barvinok_sys::isl_basic_set_free(self.handle.as_ptr()) };
    }
}

impl Clone for BasicSet<'_> {
    fn clone(&self) -> Self {
        let handle = unsafe { barvinok_sys::isl_basic_set_copy(self.handle.as_ptr()) };
        let handle = unsafe { NonNull::new_unchecked(handle) };
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}
