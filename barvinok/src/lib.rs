use std::{marker::PhantomData, ptr::NonNull};

use barvinok_sys::isl_options_set_on_error;

pub mod ident;
pub mod list;
pub mod map;
pub mod polynomial;
mod printer;
pub mod set;
pub mod space;
pub mod value;

mod stat;

#[derive(Debug, thiserror::Error)]
#[repr(u32)]
pub enum ISLError {
    #[error("operation aborted")]
    Abort = barvinok_sys::isl_error_isl_error_abort,
    #[error("memory allocation error")]
    Alloc = barvinok_sys::isl_error_isl_error_alloc,
    #[error("unknown error")]
    Unknown = barvinok_sys::isl_error_isl_error_unknown,
    #[error("internal error")]
    Internal = barvinok_sys::isl_error_isl_error_internal,
    #[error("invalid argument")]
    Invalid = barvinok_sys::isl_error_isl_error_invalid,
    #[error("operation quota exceeded")]
    Quota = barvinok_sys::isl_error_isl_error_quota,
    #[error("unsupported operation")]
    Unsupported = barvinok_sys::isl_error_isl_error_unsupported,
}

impl From<barvinok_sys::isl_error> for ISLError {
    fn from(value: barvinok_sys::isl_error) -> Self {
        match value {
            barvinok_sys::isl_error_isl_error_abort => ISLError::Abort,
            barvinok_sys::isl_error_isl_error_alloc => ISLError::Alloc,
            barvinok_sys::isl_error_isl_error_unknown => ISLError::Unknown,
            barvinok_sys::isl_error_isl_error_internal => ISLError::Internal,
            barvinok_sys::isl_error_isl_error_invalid => ISLError::Invalid,
            barvinok_sys::isl_error_isl_error_quota => ISLError::Quota,
            barvinok_sys::isl_error_isl_error_unsupported => ISLError::Unsupported,
            _ => ISLError::Unknown,
        }
    }
}

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
    #[error("variable position out of bounds")]
    VariablePositionOutOfBounds,
    #[error("isl error: {0}")]
    IslError(#[from] ISLError),
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

#[allow(clippy::should_implement_trait)]
impl<'a> ContextRef<'a> {
    pub fn as_ref(&self) -> &'a Context {
        unsafe { std::mem::transmute(self) }
    }
}

impl Context {
    pub fn new() -> Self {
        let ctx = unsafe { barvinok_sys::isl_ctx_alloc() };
        unsafe {
            isl_options_set_on_error(ctx, 0);
        }
        let ctx = nonnull_or_alloc_error(ctx);

        Self(ctx)
    }
    pub fn set_max_operations(&self, max_operations: usize) {
        unsafe { barvinok_sys::isl_ctx_set_max_operations(self.0.as_ptr(), max_operations as u64) }
    }
    pub fn reset_operations(&self) {
        unsafe { barvinok_sys::isl_ctx_reset_operations(self.0.as_ptr()) }
    }
    pub fn last_error(&self) -> Option<ISLError> {
        let err = unsafe { barvinok_sys::isl_ctx_last_error(self.0.as_ptr()) };
        if err == barvinok_sys::isl_error_isl_error_none {
            None
        } else {
            Some(err.into())
        }
    }
    pub fn last_error_or_unknown(&self) -> ISLError {
        self.last_error().unwrap_or(ISLError::Unknown)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum DimType {
    Cst = barvinok_sys::isl_dim_type_isl_dim_cst,
    Param = barvinok_sys::isl_dim_type_isl_dim_param,
    In = barvinok_sys::isl_dim_type_isl_dim_in,
    Out = barvinok_sys::isl_dim_type_isl_dim_out,
    Div = barvinok_sys::isl_dim_type_isl_dim_div,
    All = barvinok_sys::isl_dim_type_isl_dim_all,
}
