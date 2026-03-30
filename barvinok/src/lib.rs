use std::{marker::PhantomData, ptr::NonNull};

use barvinok_sys::isl_options_set_on_error;

pub mod aff;
pub mod constraint;
pub mod ident;
pub mod list;
pub mod local_space;
pub mod map;
pub mod mat;
pub mod multi;
pub mod multi_aff;
pub mod multi_id;
pub mod multi_pw_aff;
pub mod multi_union_pw_aff;
pub mod multi_val;
pub mod point;
pub mod polynomial;
mod printer;
pub mod pw_aff;
pub mod pw_multi_aff;
pub mod set;
pub mod space;
pub mod union_map;
pub mod union_pw_aff;
pub mod union_pw_multi_aff;
pub mod union_set;
pub mod value;
pub mod vec;

mod stat;

#[derive(Debug, thiserror::Error)]
#[repr(u32)]
pub enum ISLErrorKind {
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

#[derive(Debug, thiserror::Error)]
#[error("isl error({kind:?}): {message}")]
pub struct ISLError {
    kind: ISLErrorKind,
    message: String,
}

impl From<barvinok_sys::isl_error> for ISLErrorKind {
    fn from(value: barvinok_sys::isl_error) -> Self {
        match value {
            barvinok_sys::isl_error_isl_error_abort => ISLErrorKind::Abort,
            barvinok_sys::isl_error_isl_error_alloc => ISLErrorKind::Alloc,
            barvinok_sys::isl_error_isl_error_unknown => ISLErrorKind::Unknown,
            barvinok_sys::isl_error_isl_error_internal => ISLErrorKind::Internal,
            barvinok_sys::isl_error_isl_error_invalid => ISLErrorKind::Invalid,
            barvinok_sys::isl_error_isl_error_quota => ISLErrorKind::Quota,
            barvinok_sys::isl_error_isl_error_unsupported => ISLErrorKind::Unsupported,
            _ => ISLErrorKind::Unknown,
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

pub(crate) trait FromRawIsl<'a>: Sized {
    type Handle;

    unsafe fn from_raw_nonnull(handle: NonNull<Self::Handle>) -> Self;
}

#[repr(transparent)]
pub struct Context(NonNull<barvinok_sys::isl_ctx>);

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ContextRef<'a>(
    NonNull<barvinok_sys::isl_ctx>,
    PhantomData<*mut &'a Context>,
);

#[allow(clippy::should_implement_trait)]
impl<'a> ContextRef<'a> {
    pub fn set_max_operations(&self, max_operations: usize) {
        unsafe { barvinok_sys::isl_ctx_set_max_operations(self.0.as_ptr(), max_operations as u64) }
    }
    pub fn get_max_operations(&self) -> usize {
        unsafe { barvinok_sys::isl_ctx_get_max_operations(self.0.as_ptr()) as usize }
    }
    pub fn reset_operations(&self) {
        unsafe { barvinok_sys::isl_ctx_reset_operations(self.0.as_ptr()) }
    }
    pub fn last_error(&self) -> Option<ISLError> {
        let err = unsafe { barvinok_sys::isl_ctx_last_error(self.0.as_ptr()) };
        if err == barvinok_sys::isl_error_isl_error_none {
            None
        } else {
            let message = unsafe {
                let message = barvinok_sys::isl_ctx_last_error_msg(self.0.as_ptr());
                std::ffi::CStr::from_ptr(message)
                    .to_str()
                    .unwrap_or("unknown error")
                    .to_string()
            };
            Some(ISLError {
                kind: err.into(),
                message,
            })
        }
    }
    pub fn last_error_or_unknown(&self) -> ISLError {
        self.last_error().unwrap_or(ISLError {
            kind: ISLErrorKind::Unknown,
            message: "unknown error".to_string(),
        })
    }
}

impl Context {
    pub fn new() -> Self {
        let ctx = unsafe { barvinok_sys::isl_ctx_alloc() };
        unsafe {
            isl_options_set_on_error(ctx, 1);
        }
        let ctx = nonnull_or_alloc_error(ctx);

        Self(ctx)
    }
    /// # Safety
    /// This function is for using ISL's original parsing method.
    pub unsafe fn from_args<'a>(args: impl Iterator<Item = &'a str>) -> Result<Self> {
        let def = unsafe { &barvinok_sys::barvinok_options_args as *const _ as *mut _ };
        let options = unsafe { barvinok_sys::barvinok_options_new_with_defaults() };
        let ctx = unsafe { barvinok_sys::isl_ctx_alloc_with_options(def, options as _) };
        let this = std::env::current_exe()
            .map(|x| x.to_string_lossy().into_owned())
            .unwrap_or_default();
        let this = std::ffi::CString::new(this)?.into_raw();
        let argv = [Ok(this)]
            .into_iter()
            .chain(args.map(|arg| {
                let cstr = std::ffi::CString::new(arg)?;
                Ok(cstr.into_raw())
            }))
            .collect::<Result<Vec<_>>>()?;
        let argc = argv.len() as i32;
        unsafe {
            barvinok_sys::isl_ctx_parse_options(ctx, argc, argv.as_ptr() as _, 3);
            isl_options_set_on_error(ctx, 1);
        };
        for arg in argv {
            unsafe { drop(std::ffi::CString::from_raw(arg)) };
        }
        let ctx = nonnull_or_alloc_error(ctx);
        Ok(Self(ctx))
    }
    pub fn scope<F, T>(&self, f: F) -> T
    where
        F: for<'a> FnOnce(ContextRef<'a>) -> T,
    {
        let ctx = ContextRef(self.0, PhantomData);
        f(ctx)
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

macro_rules! impl_isl_handle {
    ($($tt:tt)*) => {
        barvinok_macros::impl_isl_handle!(runtime = crate, sys = barvinok_sys, $($tt)*);
    };
}

macro_rules! isl_ctor {
    ($($tt:tt)*) => {
        barvinok_macros::isl_ctor!(runtime = crate, sys = barvinok_sys, $($tt)*);
    };
}

macro_rules! isl_transform {
    ($($tt:tt)*) => {
        barvinok_macros::isl_transform!(runtime = crate, sys = barvinok_sys, $($tt)*);
    };
}

macro_rules! isl_transform_opt {
    ($($tt:tt)*) => {
        barvinok_macros::isl_transform_opt!(runtime = crate, sys = barvinok_sys, $($tt)*);
    };
}

macro_rules! isl_project {
    ($($tt:tt)*) => {
        barvinok_macros::isl_project!(runtime = crate, sys = barvinok_sys, $($tt)*);
    };
}

macro_rules! isl_project_opt {
    ($($tt:tt)*) => {
        barvinok_macros::isl_project_opt!(runtime = crate, sys = barvinok_sys, $($tt)*);
    };
}

macro_rules! isl_flag {
    ($($tt:tt)*) => {
        barvinok_macros::isl_flag!(runtime = crate, sys = barvinok_sys, $($tt)*);
    };
}

macro_rules! isl_str_opt {
    ($($tt:tt)*) => {
        barvinok_macros::isl_str_opt!(runtime = crate, sys = barvinok_sys, $($tt)*);
    };
}

macro_rules! isl_str {
    ($($tt:tt)*) => {
        barvinok_macros::isl_str!(runtime = crate, sys = barvinok_sys, $($tt)*);
    };
}

macro_rules! isl_size_opt {
    ($($tt:tt)*) => {
        barvinok_macros::isl_size_opt!(runtime = crate, sys = barvinok_sys, $($tt)*);
    };
}

macro_rules! isl_size {
    ($($tt:tt)*) => {
        barvinok_macros::isl_size!(runtime = crate, sys = barvinok_sys, $($tt)*);
    };
}

pub(crate) use impl_isl_handle;
pub(crate) use isl_ctor;
pub(crate) use isl_flag;
pub(crate) use isl_project;
pub(crate) use isl_project_opt;
pub(crate) use isl_size;
pub(crate) use isl_size_opt;
pub(crate) use isl_str;
pub(crate) use isl_str_opt;
pub(crate) use isl_transform;
pub(crate) use isl_transform_opt;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Context, aff::Affine, constraint::Constraint, ident::Ident, local_space::LocalSpace,
        map::Map, set::Set, space::Space,
    };

    #[test]
    fn test_triangular_access() -> anyhow::Result<()> {
        // for i in 0 .. n
        //     for j in 0 .. i
        //         for k in 0 .. j
        //            access A[k]
        let ctx = unsafe { Context::from_args(["--verbose"].into_iter())? };
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 3)?;
            let local_space = LocalSpace::try_from(space.clone())?;
            let i_ge_0 = Constraint::new_inequality(local_space.clone())?
                .set_coefficient_si(DimType::Out, 0, 1)?
                .set_constant_si(0)?;
            let i_lt_n = Constraint::new_inequality(local_space.clone())?
                .set_coefficient_si(DimType::Param, 0, 1)?
                .set_coefficient_si(DimType::Out, 0, -1)?
                .set_constant_si(-1)?;
            let j_ge_0 = Constraint::new_inequality(local_space.clone())?
                .set_coefficient_si(DimType::Out, 1, 1)?
                .set_constant_si(0)?;
            let j_lt_i = Constraint::new_inequality(local_space.clone())?
                .set_coefficient_si(DimType::Out, 0, 1)?
                .set_coefficient_si(DimType::Out, 1, -1)?
                .set_constant_si(-1)?;
            let k_ge_0 = Constraint::new_inequality(local_space.clone())?
                .set_coefficient_si(DimType::Out, 2, 1)?
                .set_constant_si(0)?;
            let k_lt_j = Constraint::new_inequality(local_space.clone())?
                .set_coefficient_si(DimType::Out, 1, 1)?
                .set_coefficient_si(DimType::Out, 2, -1)?
                .set_constant_si(-1)?;
            let is = Set::universe(space.clone())?
                .add_constraint(i_ge_0)?
                .add_constraint(i_lt_n)?
                .add_constraint(j_ge_0)?
                .add_constraint(j_lt_i)?
                .add_constraint(k_ge_0)?
                .add_constraint(k_lt_j)?
                .set_dim_name(DimType::Param, 0, "n")?
                .set_dim_name(DimType::Out, 0, "i")?
                .set_dim_name(DimType::Out, 1, "j")?
                .set_dim_name(DimType::Out, 2, "k")?;
            println!("IS {:?}", is);
            let card = is.clone().cardinality()?;
            println!("card IS := {:?}", card);
            let space = is.get_space()?;
            let local_space = LocalSpace::try_from(space.clone())?;
            let array_id = Ident::new(ctx, "A")?;
            let access = Affine::var_on_domain(local_space, DimType::Out, 2)?;
            let access = Map::try_from(access)?
                .set_tuple_id(DimType::Out, array_id)?
                .intersect_domain(is.clone())?;
            let lt = Map::lex_lt(space.clone())?;
            let lt = lt
                .intersect_domain(is.clone())?
                .intersect_range(is.clone())?;
            let le = Map::lex_le(space.clone())?;
            let le = le
                .intersect_domain(is.clone())?
                .intersect_range(is.clone())?;
            let access_rev = access.clone().reverse()?;
            println!("Access: {:?}", access);
            println!("Access Rev: {:?}", access_rev);
            println!("LT: {:?}", lt);
            println!("LE: {:?}", le);
            let access_compose = access.apply_range(access_rev)?;
            println!("Access Compose: {:?}", access_compose);
            let access_then_access = access_compose.intersect(lt.clone())?;
            println!("Access Then Access: {:?}", access_then_access);
            let immediate_next = access_then_access.lexmin()?;
            println!("Immediate Next: {:?}", immediate_next);
            let immediate_prev = immediate_next.reverse()?;
            println!("Immediate Prev: {:?}", immediate_prev);
            let after = immediate_prev.apply_range(lt)?;
            println!("After: {:?}", after);
            let ri = after.intersect(le.reverse()?)?;
            println!("RI Set: {:?}", ri);
            let ri_values = ri.cardinality()?;
            println!("RI Values: {:?}", ri_values);
            ri_values.foreach_piece(|poly, set| {
                println!("Poly: {:?}", poly);
                println!("Set: {:?}", set);
                Ok(())
            })?;
            Ok(())
        })
    }

    #[test]
    fn test_from_args() {
        unsafe { Context::from_args(["--verbose"].into_iter()).unwrap() };
    }
}
