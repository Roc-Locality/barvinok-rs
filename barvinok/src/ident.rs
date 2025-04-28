use crate::{Context, impl_isl_handle, nonnull_or_alloc_error};
use std::{any::Any, ptr::NonNull};

impl_isl_handle!(Ident, id);

type CastFn = fn(NonNull<std::ffi::c_void>) -> NonNull<dyn Any>;

#[repr(C)]
struct UserData<T> {
    get_as_any: CastFn,
    data: T,
}

impl<'a> Ident<'a> {
    pub fn new(ctx: &'a Context, name: &str) -> Result<Self, crate::Error> {
        let cstring = std::ffi::CString::new(name)?;

        let handle = unsafe {
            barvinok_sys::isl_id_alloc(ctx.0.as_ptr(), cstring.as_ptr(), std::ptr::null_mut())
        };

        let handle = nonnull_or_alloc_error(handle);

        Ok(Self {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn new_with_user<T: Any>(
        ctx: &'a Context,
        name: &str,
        user_data: T,
    ) -> Result<Self, crate::Error> {
        fn get_as_any<T: Any>(ptr: NonNull<std::ffi::c_void>) -> NonNull<dyn Any> {
            let user_data = ptr.cast::<UserData<T>>();
            let user_data = unsafe { &user_data.as_ref().data };
            NonNull::from(user_data)
        }
        unsafe extern "C" fn drop<T>(ptr: *mut std::ffi::c_void) {
            let user_data = ptr as *mut UserData<T>;
            let user_data = unsafe { Box::from_raw(user_data) };
            std::mem::drop(user_data);
        }

        let cstring = std::ffi::CString::new(name)?;

        let user_data = Box::new(UserData {
            get_as_any: get_as_any::<T>,
            data: user_data,
        });

        let user_data = Box::into_raw(user_data) as *mut std::ffi::c_void;

        let handle =
            unsafe { barvinok_sys::isl_id_alloc(ctx.0.as_ptr(), cstring.as_ptr(), user_data) };

        unsafe {
            barvinok_sys::isl_id_set_free_user(handle, Some(drop::<T>));
        }

        let handle = nonnull_or_alloc_error(handle);

        Ok(Self {
            handle,
            marker: std::marker::PhantomData,
        })
    }

    pub fn get_user_ref(&self) -> Option<&dyn Any> {
        let user_data = unsafe { barvinok_sys::isl_id_get_user(self.handle.as_ptr()) };
        NonNull::new(user_data).map(|ptr| {
            let cast_fn = ptr.cast::<CastFn>();
            let cast_fn = unsafe { cast_fn.as_ref() };
            let user_data = cast_fn(ptr);
            unsafe { user_data.as_ref() }
        })
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
        let ident = Ident::new_with_user(&ctx, "x", 42).unwrap();
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
            let user_data = Test(drop_flag.clone());
            let ident = Ident::new_with_user(&ctx, "x", user_data).unwrap();
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
