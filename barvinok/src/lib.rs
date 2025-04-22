use std::{marker::PhantomData, ptr::NonNull};

pub mod ident;
pub mod list;
pub mod polynomial;
mod printer;
pub mod space;
pub mod value;

mod stat;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("expected an integer value, got rational or nan")]
    NonIntegralValue,
    #[error("invalid string format")]
    ParseError,
    #[error("nul character in string")]
    NulError(#[from] std::ffi::NulError),
    #[error("isl string is not valid utf8")]
    Utf8Error(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

fn nonnull_or_alloc_error<T>(ptr: *mut T) -> NonNull<T> {
    // We don't know the exact layout of T, it is likely to be an opaque ZST.
    // This is the best we can do.
    NonNull::new(ptr).unwrap_or_else(|| {
        std::alloc::handle_alloc_error(std::alloc::Layout::new::<T>());
    })
}

#[repr(transparent)]
pub struct Context(NonNull<barvinok_sys::isl_ctx>);

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(transparent)]
pub struct ContextRef<'a>(NonNull<barvinok_sys::isl_ctx>, PhantomData<*mut &'a ()>);

impl<'a> ContextRef<'a> {
    pub fn as_ref(&self) -> &'a Context {
        unsafe { std::mem::transmute(self) }
    }
}

impl Context {
    pub fn new() -> Self {
        let ctx = unsafe { barvinok_sys::isl_ctx_alloc() };
        let ctx = nonnull_or_alloc_error(ctx);
        Self(ctx)
    }
    pub fn set_max_operations(&self, max_operations: usize) {
        unsafe { barvinok_sys::isl_ctx_set_max_operations(self.0.as_ptr(), max_operations as u64) }
    }
    pub fn reset_operations(&self) {
        unsafe { barvinok_sys::isl_ctx_reset_operations(self.0.as_ptr()) }
    }
}
