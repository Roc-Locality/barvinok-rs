use barvinok_sys::{isl_bool, isl_size};

use crate::{ContextRef, ISLError};

pub(crate) fn isl_bool_to_optional_bool(b: isl_bool) -> Option<bool> {
    match b.cmp(&0) {
        std::cmp::Ordering::Less => None,
        std::cmp::Ordering::Equal => Some(false),
        std::cmp::Ordering::Greater => Some(true),
    }
}

pub(crate) fn isl_size_to_optional_u32(s: isl_size) -> Option<u32> {
    if s < 0 { None } else { Some(s as u32) }
}

pub(crate) trait ContextResult<T> {
    fn context_result<'a>(self, ctx: ContextRef<'a>) -> Result<T, ISLError>;
}

impl ContextResult<bool> for Option<bool> {
    fn context_result<'a>(self, ctx: ContextRef<'a>) -> Result<bool, ISLError> {
        match self {
            Some(x) => Ok(x),
            None => Err(ctx.as_ref().last_error_or_unknown()),
        }
    }
}

impl ContextResult<u32> for Option<u32> {
    fn context_result<'a>(self, ctx: ContextRef<'a>) -> Result<u32, ISLError> {
        match self {
            Some(x) => Ok(x),
            None => Err(ctx.as_ref().last_error_or_unknown()),
        }
    }
}
