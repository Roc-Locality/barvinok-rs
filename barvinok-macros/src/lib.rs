#![doc = include_str!("../README.md")]
#![allow(clippy::missing_safety_doc)]

#[doc(hidden)]
#[macro_export]
macro_rules! __isl_get_access {
    ([trivial] $val:ident) => {
        $val
    };
    ([managed] $val:ident) => {
        $val.handle.as_ptr()
    };
    ([ref] $val:ident) => {
        $val.handle.as_ptr()
    };
    ([cast($target:ty)] $val:ident) => {
        $val as $target
    };
    ([str] $val:ident) => {
        $val.as_ptr()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __isl_take_arg {
    ([trivial] $val:ident) => {
        $val
    };
    ([managed] $val:ident) => {
        std::mem::ManuallyDrop::new($val)
    };
    ([ref] $val:ident) => {
        $val
    };
    ([cast($target:ty)] $val:ident) => {
        $val as $target
    };
    ([str] $val:ident) => {
        std::ffi::CString::new($val)?
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __isl_take_arg_opt {
    ([trivial] $val:ident) => {
        $val
    };
    ([managed] $val:ident) => {
        std::mem::ManuallyDrop::new($val)
    };
    ([ref] $val:ident) => {
        $val
    };
    ([cast($target:ty)] $val:ident) => {
        $val as $target
    };
    ([str] $val:ident) => {
        std::ffi::CString::new($val).ok()?
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_isl_dump {
    (runtime = $runtime:ident, sys = $sys:ident, dump, $RustType:ident, $cname:ident) => {
        paste::paste! {
            pub fn dump(&self) {
                unsafe { $sys::[<isl_ $cname _dump>](self.handle.as_ptr()) };
            }
        }
    };
    (runtime = $runtime:ident, sys = $sys:ident, nodump, $RustType:ident, $cname:ident) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_isl_print {
    (runtime = $runtime:ident, sys = $sys:ident, direct, $RustType:ident, $cname:ident) => {
        paste::paste! {
            unsafe impl<'a> $runtime::printer::ISLPrint<'a> for $RustType<'a> {
                type Handle = $sys::[<isl_ $cname>];

                fn handle(&self) -> *mut Self::Handle {
                    self.handle.as_ptr()
                }

                const TO_STRING_FFI: unsafe fn(*mut Self::Handle) -> *mut std::ffi::c_char =
                    |handle| unsafe { $sys::[<isl_ $cname _to_str>](handle) };
            }

            impl std::fmt::Debug for $RustType<'_> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let wrapper = $runtime::printer::FmtWrapper::new(self);
                    std::fmt::Debug::fmt(&wrapper, f)
                }
            }
        }
    };
    (runtime = $runtime:ident, sys = $sys:ident, printer, $RustType:ident, $cname:ident) => {
        paste::paste! {
            unsafe impl<'a> $runtime::printer::ISLPrint<'a> for $RustType<'a> {
                type Handle = $sys::[<isl_ $cname>];

                fn handle(&self) -> *mut Self::Handle {
                    self.handle.as_ptr()
                }

                const TO_STRING_FFI: unsafe fn(*mut Self::Handle) -> *mut std::ffi::c_char =
                    |handle| unsafe {
                        let ctx = $sys::[<isl_ $cname _get_ctx>](handle);
                        let Some(ctx) = std::ptr::NonNull::new(ctx) else {
                            return std::ptr::null_mut();
                        };
                        let Some(printer) = $runtime::printer::Printer::new(ctx) else {
                            return std::ptr::null_mut();
                        };
                        let Some(printer) = printer.transform(
                            $sys::[<isl_printer_print_ $cname>],
                            handle,
                        ) else {
                            return std::ptr::null_mut();
                        };
                        $sys::isl_printer_get_str(printer.as_ptr())
                    };
            }

            impl std::fmt::Debug for $RustType<'_> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let wrapper = $runtime::printer::FmtWrapper::new(self);
                    std::fmt::Debug::fmt(&wrapper, f)
                }
            }
        }
    };
    (runtime = $runtime:ident, sys = $sys:ident, none, $RustType:ident, $cname:ident) => {};
}

#[macro_export]
macro_rules! impl_isl_handle {
    (runtime = $runtime:ident, sys = $sys:ident, [noprint nodump] $RustType:ident, $cname:ident) => {
        $crate::impl_isl_handle!(@impl runtime = $runtime, sys = $sys, none, nodump, $RustType, $cname);
    };
    (runtime = $runtime:ident, sys = $sys:ident, [printer nodump] $RustType:ident, $cname:ident) => {
        $crate::impl_isl_handle!(@impl runtime = $runtime, sys = $sys, printer, nodump, $RustType, $cname);
    };
    (runtime = $runtime:ident, sys = $sys:ident, [nodump] $RustType:ident, $cname:ident) => {
        $crate::impl_isl_handle!(@impl runtime = $runtime, sys = $sys, direct, nodump, $RustType, $cname);
    };
    (runtime = $runtime:ident, sys = $sys:ident, [noprint] $RustType:ident, $cname:ident) => {
        $crate::impl_isl_handle!(@impl runtime = $runtime, sys = $sys, none, dump, $RustType, $cname);
    };
    (runtime = $runtime:ident, sys = $sys:ident, [printer] $RustType:ident, $cname:ident) => {
        $crate::impl_isl_handle!(@impl runtime = $runtime, sys = $sys, printer, dump, $RustType, $cname);
    };
    (runtime = $runtime:ident, sys = $sys:ident, $RustType:ident, $cname:ident) => {
        $crate::impl_isl_handle!(@impl runtime = $runtime, sys = $sys, direct, dump, $RustType, $cname);
    };
    (@impl runtime = $runtime:ident, sys = $sys:ident, $print_mode:ident, $dump_mode:ident, $RustType:ident, $cname:ident) => {
        paste::paste! {
            #[repr(transparent)]
            pub struct $RustType<'a> {
                pub(crate) handle: std::ptr::NonNull<$sys::[<isl_ $cname>]>,
                pub(crate) marker: std::marker::PhantomData<*mut &'a ()>,
            }

            impl Clone for $RustType<'_> {
                fn clone(&self) -> Self {
                    let handle = unsafe { $sys::[<isl_ $cname _copy>](self.handle.as_ptr()) };
                    let handle = $runtime::nonnull_or_alloc_error(handle);
                    Self {
                        handle,
                        marker: std::marker::PhantomData,
                    }
                }
            }

            impl Drop for $RustType<'_> {
                fn drop(&mut self) {
                    unsafe { $sys::[<isl_ $cname _free>](self.handle.as_ptr()) };
                }
            }

            impl<'a> $RustType<'a> {
                pub fn context_ref(&self) -> $runtime::ContextRef<'a> {
                    let ctx = unsafe { $sys::[<isl_ $cname _get_ctx>](self.handle.as_ptr()) };
                    let ptr = unsafe { std::ptr::NonNull::new_unchecked(ctx) };
                    $runtime::ContextRef(ptr, std::marker::PhantomData)
                }
            }

            impl<'a> $runtime::FromRawIsl<'a> for $RustType<'a> {
                type Handle = $sys::[<isl_ $cname>];

                unsafe fn from_raw_nonnull(handle: std::ptr::NonNull<Self::Handle>) -> Self {
                    Self {
                        handle,
                        marker: std::marker::PhantomData,
                    }
                }
            }

            impl<'a> $RustType<'a> {
                $crate::__impl_isl_dump!(runtime = $runtime, sys = $sys, $dump_mode, $RustType, $cname);
            }

            $crate::__impl_isl_print!(runtime = $runtime, sys = $sys, $print_mode, $RustType, $cname);
        }
    };
}

#[macro_export]
macro_rules! isl_ctor {
    (runtime = $runtime:ident, sys = $sys:ident, $func:ident, $sys_fn:ident,
     $first_name:ident : $first_ty:ty
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            $first_name: $first_ty
            $(, $name: $ty )*
        ) -> Result<Self, $runtime::Error> {
            let ctx = $first_name.context_ref();
            let $first_name = std::mem::ManuallyDrop::new($first_name);
            $(
                let $name = $crate::__isl_take_arg!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    $first_name.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name) )*
                )
            };

            std::ptr::NonNull::new(raw)
                .ok_or_else(|| ctx.last_error_or_unknown().into())
                .map(|handle| Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
        }
    };
    (runtime = $runtime:ident, sys = $sys:ident, [ctx] $func:ident, $sys_fn:ident
        $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            ctx: $runtime::ContextRef<'a>
            $(, $name: $ty )*
        ) -> Result<Self, $runtime::Error> {
            $(
                let $name = $crate::__isl_take_arg!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    ctx.0.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name) )*
                )
            };

            std::ptr::NonNull::new(raw)
                .ok_or_else(|| ctx.last_error_or_unknown().into())
                .map(|handle| Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
        }
    };
}

#[macro_export]
macro_rules! isl_transform {
    (runtime = $runtime:ident, sys = $sys:ident, $func:ident, $sys_fn:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            self: Self
            $(, $name: $ty )*
        ) -> Result<Self, $runtime::Error> {
            let ctx = self.context_ref();
            let this = std::mem::ManuallyDrop::new(self);
            $(
                let $name = $crate::__isl_take_arg!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    this.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name))*
                )
            };

            std::ptr::NonNull::new(raw)
                .ok_or_else(|| ctx.last_error_or_unknown().into())
                .map(|handle| Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
        }
    };
    (runtime = $runtime:ident, sys = $sys:ident, [into($target:ident)] $func:ident, $sys_fn:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            self: Self
            $(, $name: $ty )*
        ) -> Result<$target<'a>, $runtime::Error> {
            let ctx = self.context_ref();
            let this = std::mem::ManuallyDrop::new(self);
            $(
                let $name = $crate::__isl_take_arg!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    this.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name) )*
                )
            };

            std::ptr::NonNull::new(raw)
                .ok_or_else(|| ctx.last_error_or_unknown().into())
                .map(|handle| $target {
                    handle,
                    marker: std::marker::PhantomData,
                })
        }
    };
    (runtime = $runtime:ident, sys = $sys:ident, [into($target:ty)] $func:ident, $sys_fn:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            self: Self
            $(, $name: $ty )*
        ) -> Result<$target, $runtime::Error> {
            let ctx = self.context_ref();
            let this = std::mem::ManuallyDrop::new(self);
            $(
                let $name = $crate::__isl_take_arg!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    this.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name) )*
                )
            };

            std::ptr::NonNull::new(raw)
                .ok_or_else(|| ctx.last_error_or_unknown().into())
                .map(|handle| unsafe {
                    <$target as $runtime::FromRawIsl<'a>>::from_raw_nonnull(handle)
                })
        }
    };
}

#[macro_export]
macro_rules! isl_transform_opt {
    (runtime = $runtime:ident, sys = $sys:ident, $func:ident, $sys_fn:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            self: Self
            $(, $name: $ty )*
        ) -> Option<Self> {
            let this = std::mem::ManuallyDrop::new(self);
            $(
                let $name = $crate::__isl_take_arg_opt!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    this.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name))*
                )
            };

            std::ptr::NonNull::new(raw).map(|handle| Self {
                handle,
                marker: std::marker::PhantomData,
            })
        }
    };
    (runtime = $runtime:ident, sys = $sys:ident, [into($target:ident)] $func:ident, $sys_fn:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            self: Self
            $(, $name: $ty )*
        ) -> Option<$target<'a>> {
            let this = std::mem::ManuallyDrop::new(self);
            $(
                let $name = $crate::__isl_take_arg_opt!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    this.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name))*
                )
            };

            std::ptr::NonNull::new(raw).map(|handle| $target {
                handle,
                marker: std::marker::PhantomData,
            })
        }
    };
    (runtime = $runtime:ident, sys = $sys:ident, [into($target:ty)] $func:ident, $sys_fn:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            self: Self
            $(, $name: $ty )*
        ) -> Option<$target> {
            let this = std::mem::ManuallyDrop::new(self);
            $(
                let $name = $crate::__isl_take_arg_opt!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    this.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name))*
                )
            };

            std::ptr::NonNull::new(raw).map(|handle| unsafe {
                <$target as $runtime::FromRawIsl<'a>>::from_raw_nonnull(handle)
            })
        }
    };
}

#[macro_export]
macro_rules! isl_project {
    (runtime = $runtime:ident, sys = $sys:ident, [into($target:ident)] $func:ident, $sys_fn:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            &self
            $(, $name: $ty )*
        ) -> Result<$target<'a>, $runtime::Error> {
            $(
                let $name = $crate::__isl_take_arg!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    self.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name) )*
                )
            };

            std::ptr::NonNull::new(raw)
                .ok_or_else(|| self.context_ref().last_error_or_unknown().into())
                .map(|handle| $target {
                    handle,
                    marker: std::marker::PhantomData,
                })
        }
    };
    (runtime = $runtime:ident, sys = $sys:ident, [into($target:ty)] $func:ident, $sys_fn:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            &self
            $(, $name: $ty )*
        ) -> Result<$target, $runtime::Error> {
            $(
                let $name = $crate::__isl_take_arg!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    self.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name) )*
                )
            };

            std::ptr::NonNull::new(raw)
                .ok_or_else(|| self.context_ref().last_error_or_unknown().into())
                .map(|handle| unsafe {
                    <$target as $runtime::FromRawIsl<'a>>::from_raw_nonnull(handle)
                })
        }
    };
}

#[macro_export]
macro_rules! isl_project_opt {
    (runtime = $runtime:ident, sys = $sys:ident, [into($target:ident)] $func:ident, $sys_fn:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            &self
            $(, $name: $ty )*
        ) -> Option<$target<'a>> {
            $(
                let $name = $crate::__isl_take_arg_opt!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    self.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name))*
                )
            };

            std::ptr::NonNull::new(raw).map(|handle| $target {
                handle,
                marker: std::marker::PhantomData,
            })
        }
    };
    (runtime = $runtime:ident, sys = $sys:ident, [into($target:ty)] $func:ident, $sys_fn:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            &self
            $(, $name: $ty )*
        ) -> Option<$target> {
            $(
                let $name = $crate::__isl_take_arg_opt!([$kind $(($param))*] $name);
            )*

            let raw = unsafe {
                $sys::$sys_fn(
                    self.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name))*
                )
            };

            std::ptr::NonNull::new(raw).map(|handle| unsafe {
                <$target as $runtime::FromRawIsl<'a>>::from_raw_nonnull(handle)
            })
        }
    };
}

#[macro_export]
macro_rules! isl_flag {
    (runtime = $runtime:ident, sys = $sys:ident, $isl_func:ident => $fn_name:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)?) => {
        pub fn $fn_name(&self $(, $name: $ty )*) -> Result<bool, $runtime::Error> {
            $(
                let $name = $crate::__isl_take_arg!([$kind $(($param))*] $name);
            )*
            let flag = unsafe {
                $sys::$isl_func(
                    self.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name))*
                )
            };
            $runtime::stat::isl_bool_to_optional_bool(flag)
                .ok_or_else(|| self.context_ref().last_error_or_unknown().into())
        }
    };
}

#[macro_export]
macro_rules! isl_str_opt {
    (runtime = $runtime:ident, sys = $sys:ident, $isl_func:ident => $fn_name:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)?) => {
        pub fn $fn_name(&self $(, $name: $ty )*) -> Result<Option<&str>, $runtime::Error> {
            $(
                let $name = $crate::__isl_take_arg!([$kind $(($param))*] $name);
            )*
            let ptr = unsafe {
                $sys::$isl_func(
                    self.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name))*
                )
            };
            if ptr.is_null() {
                return Ok(None);
            }
            let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
            Ok(Some(cstr.to_str()?))
        }
    };
}

#[macro_export]
macro_rules! isl_str {
    (runtime = $runtime:ident, sys = $sys:ident, $isl_func:ident => $fn_name:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)?) => {
        pub fn $fn_name(&self $(, $name: $ty )*) -> Result<&str, $runtime::Error> {
            $(
                let $name = $crate::__isl_take_arg!([$kind $(($param))*] $name);
            )*
            let ptr = unsafe {
                $sys::$isl_func(
                    self.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name))*
                )
            };
            if ptr.is_null() {
                return Err(self.context_ref().last_error_or_unknown().into());
            }
            let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
            Ok(cstr.to_str()?)
        }
    };
}

#[macro_export]
macro_rules! isl_size_opt {
    (runtime = $runtime:ident, sys = $sys:ident, $isl_func:ident => $fn_name:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)?) => {
        pub fn $fn_name(&self $(, $name: $ty )*) -> Option<u32> {
            $(
                let $name = $crate::__isl_take_arg_opt!([$kind $(($param))*] $name);
            )*
            let size = unsafe {
                $sys::$isl_func(
                    self.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name))*
                )
            };
            if size < 0 {
                None
            } else {
                Some(size as u32)
            }
        }
    };
}

#[macro_export]
macro_rules! isl_size {
    (runtime = $runtime:ident, sys = $sys:ident, $isl_func:ident => $fn_name:ident
     $(, [$kind:ident $(($param:ty))?] $name:ident : $ty:ty )* $(,)?) => {
        pub fn $fn_name(&self $(, $name: $ty )*) -> Result<u32, $runtime::Error> {
            $(
                let $name = $crate::__isl_take_arg!([$kind $(($param))*] $name);
            )*
            let size = unsafe {
                $sys::$isl_func(
                    self.handle.as_ptr()
                    $(, $crate::__isl_get_access!([$kind $(($param))*] $name))*
                )
            };
            $runtime::stat::isl_size_to_optional_u32(size)
                .ok_or_else(|| self.context_ref().last_error_or_unknown().into())
        }
    };
}
