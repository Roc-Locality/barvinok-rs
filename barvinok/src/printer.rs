use std::{
    ffi::{CString, c_char},
    fmt::Debug,
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
};

/// # Safety
/// APIs must meet with ISL's assumptions.
pub unsafe trait ISLPrint<'a> {
    type Handle;
    const TO_STRING_FFI: unsafe fn(*mut Self::Handle) -> *mut c_char;
    fn handle(&self) -> *mut Self::Handle;
}

pub(crate) struct FmtWrapper<'a, 'b, T: ISLPrint<'b>>(&'a T, std::marker::PhantomData<&'b ()>);

impl<'a, 'b, T> FmtWrapper<'a, 'b, T>
where
    T: ISLPrint<'b>,
{
    pub fn new(handle: &'a T) -> Self {
        Self(handle, std::marker::PhantomData)
    }
}

#[derive(Debug)]
struct MallocCString(ManuallyDrop<CString>);

impl MallocCString {
    unsafe fn from_raw(ptr: *mut c_char) -> Self {
        let c_string = unsafe { CString::from_raw(ptr) };
        MallocCString(ManuallyDrop::new(c_string))
    }
}

impl Deref for MallocCString {
    type Target = CString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for MallocCString {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.0.as_ptr();
            libc::free(ptr as *mut libc::c_void);
        }
    }
}

impl<'b, T> std::fmt::Debug for FmtWrapper<'_, 'b, T>
where
    T: ISLPrint<'b>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c_string = unsafe { (T::TO_STRING_FFI)(self.0.handle()) };
        if c_string.is_null() {
            return Err(std::fmt::Error);
        }
        let mstring = unsafe { MallocCString::from_raw(c_string) };
        let string = mstring.to_string_lossy();
        f.write_str(&string)
    }
}

impl<'b, T> std::fmt::Display for FmtWrapper<'_, 'b, T>
where
    T: ISLPrint<'b>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[repr(transparent)]
pub(crate) struct Printer(NonNull<barvinok_sys::isl_printer>);
impl Printer {
    pub(crate) unsafe fn new(ctx: NonNull<barvinok_sys::isl_ctx>) -> Option<Self> {
        let printer = unsafe { barvinok_sys::isl_printer_to_str(ctx.as_ptr()) };
        NonNull::new(printer).map(Self)
    }
    pub(crate) fn as_ptr(&self) -> *mut barvinok_sys::isl_printer {
        self.0.as_ptr()
    }
    pub(crate) unsafe fn transform<T>(
        self,
        func: unsafe extern "C" fn(
            *mut barvinok_sys::isl_printer,
            T,
        ) -> *mut barvinok_sys::isl_printer,
        data: T,
    ) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let printer = unsafe { func(this.0.as_ptr(), data) };
        NonNull::new(printer).map(Self)
    }
}

impl Drop for Printer {
    fn drop(&mut self) {
        unsafe { barvinok_sys::isl_printer_free(self.0.as_ptr()) };
    }
}

#[macro_export]
macro_rules! impl_isl_print {
    ($Rust:ident, $isl:ident) => {
        unsafe impl<'a> $crate::printer::ISLPrint<'a> for $Rust<'a> {
            type Handle = paste::paste! { barvinok_sys::[<isl_ $isl>] };
            fn handle(&self) -> *mut Self::Handle {
                self.handle.as_ptr()
            }

            const TO_STRING_FFI: unsafe fn(handle: *mut Self::Handle) -> *mut std::ffi::c_char =
                |handle| unsafe {
                    paste::paste! { barvinok_sys::[<isl_ $isl _to_str>](handle)}
                };
        }

        impl std::fmt::Debug for $Rust<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let wrapper = $crate::printer::FmtWrapper::new(self);
                std::fmt::Debug::fmt(&wrapper, f)
            }
        }
    };

    // there are types without isl_XXX_to_str function.
    // in those cases, let's print the type to a in-memory file
    // and then read the file to get the string
    ([printer] $Rust:ident, $isl:ident) => {
        unsafe impl<'a> $crate::printer::ISLPrint<'a> for $Rust<'a> {
            type Handle = paste::paste! { barvinok_sys::[<isl_ $isl>] };
            fn handle(&self) -> *mut Self::Handle {
                self.handle.as_ptr()
            }
            const TO_STRING_FFI: unsafe fn(handle: *mut Self::Handle) -> *mut std::ffi::c_char =
                |handle| unsafe {
                    let ctx = paste::paste! { barvinok_sys::[<isl_ $isl _get_ctx>](handle)};
                    let Some(ctx) = NonNull::new(ctx) else {
                        return std::ptr::null_mut();
                    };
                    let Some(printer) = $crate::printer::Printer::new(ctx) else {
                        return std::ptr::null_mut();
                    };
                    let Some(printer) = printer.transform(
                        paste::paste! { barvinok_sys::[<isl_printer_print_ $isl>]},
                        handle,
                    ) else {
                        return std::ptr::null_mut();
                    };
                    barvinok_sys::isl_printer_get_str(printer.as_ptr())
                };
        }

        impl std::fmt::Debug for $Rust<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let wrapper = $crate::printer::FmtWrapper::new(self);
                std::fmt::Debug::fmt(&wrapper, f)
            }
        }
    };
}
