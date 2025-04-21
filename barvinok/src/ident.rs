use crate::printer::ISLPrint;
use crate::{Context, ContextRef, nonnull_or_alloc_error};
use std::fmt::Debug;
use std::{any::Any, ptr::NonNull};

#[repr(transparent)]
pub struct Ident<'a> {
    pub(crate) handle: NonNull<barvinok_sys::isl_id>,
    pub(crate) marker: std::marker::PhantomData<*mut &'a ()>,
}

impl<'a> Ident<'a> {
    pub fn new(ctx: &'a Context, name: &str) -> Result<Self, crate::Error> {
        Self::new_with_user(ctx, name, None)
    }
    pub fn new_with_user(
        ctx: &'a Context,
        name: &str,
        user_data: Option<Box<dyn Any>>,
    ) -> Result<Self, crate::Error> {
        let user_data = user_data
            .map(Box::new)
            .map(Box::into_raw)
            .unwrap_or_else(std::ptr::null_mut);

        let cstring = std::ffi::CString::new(name)?;

        let handle = unsafe {
            barvinok_sys::isl_id_alloc(
                ctx.0.as_ptr(),
                cstring.as_ptr(),
                user_data as *mut std::ffi::c_void,
            )
        };

        unsafe extern "C" fn cleanup_user_data(user_data: *mut std::ffi::c_void) {
            std::mem::drop(unsafe { Box::from_raw(user_data as *mut Box<dyn Any>) });
        }

        if !user_data.is_null() {
            unsafe { barvinok_sys::isl_id_set_free_user(handle, Some(cleanup_user_data)) };
        }

        let handle = nonnull_or_alloc_error(handle);

        Ok(Self {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn get_user_ref(&self) -> Option<&dyn Any> {
        let user_data = unsafe { barvinok_sys::isl_id_get_user(self.handle.as_ptr()) };
        if user_data.is_null() {
            None
        } else {
            Some(unsafe { &**(user_data as *const Box<dyn Any>) })
        }
    }
    pub fn get_user_as<T: Any>(&self) -> Option<&T> {
        self.get_user_ref()
            .and_then(|user_data| user_data.downcast_ref::<T>())
    }
    pub fn name(&self) -> Result<&str, crate::Error> {
        let cstr = unsafe { barvinok_sys::isl_id_get_name(self.handle.as_ptr()) };
        let cstr = unsafe { std::ffi::CStr::from_ptr(cstr) };
        Ok(cstr.to_str()?)
    }

    pub fn context_ref(&self) -> ContextRef<'a> {
        let ctx = unsafe { barvinok_sys::isl_id_get_ctx(self.handle.as_ptr()) };
        let ctx = unsafe { NonNull::new_unchecked(ctx) };
        ContextRef(ctx, std::marker::PhantomData)
    }
    pub fn dump(&self) {
        unsafe {
            barvinok_sys::isl_id_dump(self.handle.as_ptr());
        }
    }
}

impl Clone for Ident<'_> {
    fn clone(&self) -> Self {
        let handle = unsafe { barvinok_sys::isl_id_copy(self.handle.as_ptr()) };
        let handle = unsafe { NonNull::new_unchecked(handle) };
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl Drop for Ident<'_> {
    fn drop(&mut self) {
        unsafe { barvinok_sys::isl_id_free(self.handle.as_ptr()) };
    }
}

impl<'a> ISLPrint<'a> for Ident<'a> {
    type Handle = barvinok_sys::isl_id;

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
        unsafe { barvinok_sys::isl_printer_print_id(printer, handle) }
    }
}

impl Debug for Ident<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let wrapper = crate::printer::FmtWrapper::new(self);
        Debug::fmt(&wrapper, f)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use super::*;
    use crate::Context;
    use std::sync::Arc;

    #[test]
    fn test_ident() {
        let ctx = Context::new();
        let ident = Ident::new(&ctx, "x").unwrap();
        assert_eq!(ident.name().unwrap(), "x");
        assert_eq!(ident.context_ref().0.as_ptr(), ctx.0.as_ptr());
        println!("{:?}", ident);
    }

    #[test]
    fn test_ident_with_user() {
        let ctx = Context::new();
        let user_data = Box::new(42);
        let ident = Ident::new_with_user(&ctx, "x", Some(user_data)).unwrap();
        assert_eq!(ident.name().unwrap(), "x");
        assert_eq!(ident.context_ref().0.as_ptr(), ctx.0.as_ptr());
        assert_eq!(
            ident.get_user_ref().unwrap().downcast_ref::<i32>(),
            Some(&42)
        );
    }

    #[test]
    fn test_ident_lifetime() {
        let ctx = Context::new();
        let drop_flag = Arc::new(AtomicBool::new(false));
        struct Test(Arc<AtomicBool>);
        impl Drop for Test {
            fn drop(&mut self) {
                self.0.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
        {
            let user_data = Box::new(Test(drop_flag.clone()));
            let ident = Ident::new_with_user(&ctx, "x", Some(user_data)).unwrap();
            let ident2 = ident.clone();
            let user = ident2.get_user_ref().unwrap();
            assert!(
                !user
                    .downcast_ref::<Test>()
                    .unwrap()
                    .0
                    .load(std::sync::atomic::Ordering::SeqCst)
            );
            assert!(
                !ident
                    .get_user_as::<Test>()
                    .unwrap()
                    .0
                    .load(std::sync::atomic::Ordering::SeqCst)
            );
        }
        assert!(drop_flag.load(std::sync::atomic::Ordering::SeqCst));
    }
}
