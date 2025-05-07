use crate::isl_ctor;
use crate::point::Point;
use crate::{DimType, space::Space};
use crate::{
    ident::Ident,
    impl_isl_handle, isl_flag, isl_project, isl_size, isl_transform,
    set::Set,
    stat::{isl_bool_to_optional_bool, isl_size_to_optional_u32},
    value::Value,
};
use std::mem::ManuallyDrop;
use std::{cell::Cell, ptr::NonNull};

impl_isl_handle!(QuasiPolynomial, qpolynomial);
impl_isl_handle!([noprint] Term, term);
impl_isl_handle!(PiecewiseQuasiPolynomial, pw_qpolynomial);

macro_rules! qpolynomial_constructors {
    ($($func:ident),+ $(,)?) => {
        paste::paste! {
            $(
                isl_ctor!([<$func _on_domain>], [<isl_qpolynomial_ $func _on_domain>], space : Space<'a>);
            )*
        }
    };
}

impl<'a> QuasiPolynomial<'a> {
    qpolynomial_constructors!(zero, one, infty, neginfty, nan);
    isl_project!([into(Space)] get_domain_space, isl_qpolynomial_get_domain_space);
    isl_project!([into(Space)] get_space, isl_qpolynomial_get_space);
    isl_size!(qpolynomial_dim => get_dim, [cast(u32)] dim_type: DimType);
    isl_ctor!(val_on_domain, isl_qpolynomial_val_on_domain, space: Space<'a>, [managed] value: Value<'a>);
    isl_ctor!(var_on_domain, isl_qpolynomial_var_on_domain, space: Space<'a>, [cast(u32)] dim_type: DimType, [trivial] pos: u32);
    isl_project!([into(Value)] get_constant_val, isl_qpolynomial_get_constant_val);
    isl_flag!(qpolynomial_is_zero => is_zero);
    isl_flag!(qpolynomial_is_infty => is_infty);
    isl_flag!(qpolynomial_is_neginfty => is_neginfty);
    isl_flag!(qpolynomial_is_nan => is_nan);
    isl_transform!(pow, isl_qpolynomial_pow, [cast(u32)] exp: u32);
    isl_transform!(scale_up_val, isl_qpolynomial_scale_val, [managed] value: Value<'a>);
    isl_transform!(scale_down_val, isl_qpolynomial_scale_down_val, [managed] value: Value<'a>);
    isl_transform!(domain_reverse, isl_qpolynomial_domain_reverse);
    isl_transform!(homogenize, isl_qpolynomial_homogenize);
    isl_flag!(qpolynomial_plain_is_equal => plain_is_equal, [ref] other: &QuasiPolynomial<'a>);
    isl_transform!(checked_add, isl_qpolynomial_add, [managed] other: QuasiPolynomial<'a>);
    isl_transform!(checked_sub, isl_qpolynomial_sub, [managed] other: QuasiPolynomial<'a>);
    isl_transform!(checked_mul, isl_qpolynomial_mul, [managed] other: QuasiPolynomial<'a>);
    isl_transform!(gist, isl_qpolynomial_gist, [managed] set: Set<'a>);
    isl_transform!(gist_params, isl_qpolynomial_gist_params, [managed] set: Set<'a>);
    isl_flag!(qpolynomial_involves_dims => involves_dims, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    pub fn foreach_term<F>(&self, func: F) -> Result<(), crate::Error>
    where
        F: FnMut(Term<'a>) -> Result<(), crate::Error>,
    {
        struct FuncWithState<F> {
            func: F,
            state: Cell<Result<(), crate::Error>>,
        }
        let mut func = FuncWithState {
            func,
            state: Cell::new(Ok(())),
        };
        unsafe extern "C" fn callback<'a, F>(
            term: *mut barvinok_sys::isl_term,
            user: *mut std::ffi::c_void,
        ) -> barvinok_sys::isl_stat
        where
            F: FnMut(Term<'a>) -> Result<(), crate::Error>,
        {
            let data = unsafe { &mut *(user as *mut FuncWithState<F>) };
            let term = Term {
                handle: NonNull::new(term).unwrap(),
                marker: std::marker::PhantomData,
            };
            let state = data.state.replace(Ok(()));
            data.state.set(state.and_then(|_| (data.func)(term)));
            if data.state.get_mut().is_ok() {
                barvinok_sys::isl_stat_isl_stat_ok
            } else {
                barvinok_sys::isl_stat_isl_stat_error
            }
        }
        let handle = self.handle.as_ptr();
        let res = unsafe {
            barvinok_sys::isl_qpolynomial_foreach_term(
                handle,
                Some(callback::<F>),
                &mut func as *mut FuncWithState<F> as *mut std::ffi::c_void,
            )
        };
        if res == barvinok_sys::isl_stat_isl_stat_ok {
            func.state.into_inner()
        } else {
            match func.state.into_inner() {
                Ok(()) => Err(self.context_ref().last_error_or_unknown().into()),
                Err(e) => Err(e),
            }
        }
    }
}

impl<'a> std::ops::Add for QuasiPolynomial<'a> {
    type Output = QuasiPolynomial<'a>;
    fn add(self, other: QuasiPolynomial<'a>) -> Self::Output {
        self.checked_add(other).unwrap()
    }
}

impl<'a> std::ops::Sub for QuasiPolynomial<'a> {
    type Output = QuasiPolynomial<'a>;
    fn sub(self, other: QuasiPolynomial<'a>) -> Self::Output {
        self.checked_sub(other).unwrap()
    }
}

impl<'a> std::ops::Mul for QuasiPolynomial<'a> {
    type Output = QuasiPolynomial<'a>;
    fn mul(self, other: QuasiPolynomial<'a>) -> Self::Output {
        self.checked_mul(other).unwrap()
    }
}

impl<'a> PiecewiseQuasiPolynomial<'a> {
    isl_project!([into(Space)] get_space, isl_pw_qpolynomial_get_space);
    isl_flag!(pw_qpolynomial_involves_nan => involves_nan);
    isl_flag!(pw_qpolynomial_plain_is_equal => plain_is_equal, [ref] other: &PiecewiseQuasiPolynomial<'a>);
    isl_ctor!(zero, isl_pw_qpolynomial_zero, space: Space<'a>);
    isl_flag!(pw_qpolynomial_is_zero => is_zero);
    isl_project!([into(Space)] get_domain_space, isl_pw_qpolynomial_get_domain_space);
    isl_transform!(reset_domain_space, isl_pw_qpolynomial_reset_domain_space, [managed] space: Space<'a>);
    isl_size!(pw_qpolynomial_dim => dim, [cast(u32)] dim_type: DimType);
    isl_flag!(pw_qpolynomial_involves_param_id => involves_param_id, [ref] id: Ident<'a>);
    isl_flag!(pw_qpolynomial_involves_dims => involves_dims, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    isl_flag!(pw_qpolynomial_has_equal_space => has_equal_space, [ref] other: &PiecewiseQuasiPolynomial<'a>);
    isl_transform!(set_dim_name, isl_pw_qpolynomial_set_dim_name, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [str] name: &str);
    isl_size!(pw_qpolynomial_find_dim_by_name => find_dim_by_name, [cast(u32)] dim_type: DimType, [str] name: &str);
    isl_transform!(reset_user, isl_pw_qpolynomial_reset_user);
    isl_transform!([into(Set)] domain, isl_pw_qpolynomial_domain);
    isl_transform!(intersect_domain, isl_pw_qpolynomial_intersect_domain, [managed] set: Set<'a>);
    isl_transform!(intersect_domain_wrapped_domain, isl_pw_qpolynomial_intersect_domain_wrapped_domain, [managed] set: Set<'a>);
    isl_transform!(intersect_domain_wrapped_range, isl_pw_qpolynomial_intersect_domain_wrapped_range, [managed] set: Set<'a>);
    isl_transform!(intersect_params, isl_pw_qpolynomial_intersect_params, [managed] set: Set<'a>);
    isl_transform!(subtract_domain, isl_pw_qpolynomial_subtract_domain, [managed] set: Set<'a>);
    isl_transform!(
        project_domain_on_params,
        isl_pw_qpolynomial_project_domain_on_params
    );
    isl_ctor!(from_range, isl_pw_qpolynomial_from_range, src: PiecewiseQuasiPolynomial<'a>);
    isl_transform!(drop_dims, isl_pw_qpolynomial_drop_dims, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    isl_transform!(split_dims, isl_pw_qpolynomial_split_dims, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    isl_transform!(drop_unused_params, isl_pw_qpolynomial_drop_unused_params);
    isl_transform!(checked_add, isl_pw_qpolynomial_add, [managed] other: PiecewiseQuasiPolynomial<'a>);
    isl_transform!(checked_sub, isl_pw_qpolynomial_sub, [managed] other: PiecewiseQuasiPolynomial<'a>);
    isl_transform!(add_disjoint, isl_pw_qpolynomial_add_disjoint, [managed] other: PiecewiseQuasiPolynomial<'a>);
    isl_transform!(checked_neg, isl_pw_qpolynomial_neg);
    isl_transform!(checked_mul, isl_pw_qpolynomial_mul, [managed] other: PiecewiseQuasiPolynomial<'a>);
    isl_transform!(scale_val, isl_pw_qpolynomial_scale_val, [managed] value: Value<'a>);
    isl_transform!(scale_down_val, isl_pw_qpolynomial_scale_down_val, [managed] value: Value<'a>);
    isl_transform!(pow, isl_pw_qpolynomial_pow, [trivial] exp: u32);
    isl_transform!(domain_reverse, isl_pw_qpolynomial_domain_reverse);
    isl_transform!(insert_dims, isl_pw_qpolynomial_insert_dims, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    isl_transform!(add_dims, isl_pw_qpolynomial_add_dims, [cast(u32)] dim_type: DimType, [trivial] num: u32);
    isl_transform!(move_dims, isl_pw_qpolynomial_move_dims, [cast(u32)] dst_dim_type : DimType, [trivial] dst_pos: u32, [cast(u32)] src_dim_type: DimType, [trivial] src_pos: u32, [trivial] num: u32);
    isl_transform!(fix_val, isl_pw_qpolynomial_fix_val, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [managed] value: Value<'a>);
    isl_transform!([into(Value)] eval, isl_pw_qpolynomial_eval, [managed] point: Point<'a>);
    isl_transform!([into(Value)] max, isl_pw_qpolynomial_max);
    isl_transform!([into(Value)] min, isl_pw_qpolynomial_min);
    isl_size!(pw_qpolynomial_n_piece => num_pieces);
    isl_flag!(pw_qpolynomial_isa_qpolynomial => is_qpolynomial);
    isl_transform!([into(QuasiPolynomial)] as_qpolynomial, isl_pw_qpolynomial_as_qpolynomial);
    pub fn foreach_piece<F>(&self, func: F) -> Result<(), crate::Error>
    where
        F: FnMut(QuasiPolynomial<'a>, Set<'a>) -> Result<(), crate::Error>,
    {
        struct FuncWithState<F> {
            func: F,
            state: Cell<Result<(), crate::Error>>,
        }
        let mut func = FuncWithState {
            func,
            state: Cell::new(Ok(())),
        };
        unsafe extern "C" fn callback<'a, F>(
            set: *mut barvinok_sys::isl_set,
            qpoly: *mut barvinok_sys::isl_qpolynomial,
            user: *mut std::ffi::c_void,
        ) -> barvinok_sys::isl_stat
        where
            F: FnMut(QuasiPolynomial<'a>, Set<'a>) -> Result<(), crate::Error>,
        {
            let data = unsafe { &mut *(user as *mut FuncWithState<F>) };
            let qpoly = QuasiPolynomial {
                handle: NonNull::new(qpoly).unwrap(),
                marker: std::marker::PhantomData,
            };
            let set = Set {
                handle: NonNull::new(set).unwrap(),
                marker: std::marker::PhantomData,
            };
            let state = data.state.replace(Ok(()));
            data.state.set(state.and_then(|_| (data.func)(qpoly, set)));
            if data.state.get_mut().is_ok() {
                barvinok_sys::isl_stat_isl_stat_ok
            } else {
                barvinok_sys::isl_stat_isl_stat_error
            }
        }
        let handle = self.handle.as_ptr();
        let res = unsafe {
            barvinok_sys::isl_pw_qpolynomial_foreach_piece(
                handle,
                Some(callback::<F>),
                &mut func as *mut FuncWithState<F> as *mut std::ffi::c_void,
            )
        };
        if res == barvinok_sys::isl_stat_isl_stat_ok {
            func.state.into_inner()
        } else {
            match func.state.into_inner() {
                Ok(()) => Err(self.context_ref().last_error_or_unknown().into()),
                Err(e) => Err(e),
            }
        }
    }
    pub fn every_piece<F>(&self, func: F) -> Result<bool, crate::Error>
    where
        F: FnMut(QuasiPolynomial<'a>, Set<'a>) -> Result<bool, crate::Error>,
    {
        struct FuncWithState<F> {
            func: F,
            state: Cell<Result<(), crate::Error>>,
        }
        let mut func = FuncWithState {
            func,
            state: Cell::new(Ok(())),
        };
        unsafe extern "C" fn callback<'a, F>(
            set: *mut barvinok_sys::isl_set,
            qpoly: *mut barvinok_sys::isl_qpolynomial,
            user: *mut std::ffi::c_void,
        ) -> barvinok_sys::isl_bool
        where
            F: FnMut(QuasiPolynomial<'a>, Set<'a>) -> Result<bool, crate::Error>,
        {
            let data = unsafe { &mut *(user as *mut FuncWithState<F>) };
            let qpoly = QuasiPolynomial {
                handle: NonNull::new(qpoly).unwrap(),
                marker: std::marker::PhantomData,
            };
            let set = Set {
                handle: NonNull::new(set).unwrap(),
                marker: std::marker::PhantomData,
            };
            let state = data.state.replace(Ok(()));
            match state.and_then(|_| (data.func)(qpoly, set)) {
                Ok(true) => barvinok_sys::isl_bool_isl_bool_true,
                Ok(false) => barvinok_sys::isl_bool_isl_bool_false,
                Err(e) => {
                    data.state.set(Err(e));
                    barvinok_sys::isl_bool_isl_bool_error
                }
            }
        }
        let handle = self.handle.as_ptr();
        let res = unsafe {
            barvinok_sys::isl_pw_qpolynomial_every_piece(
                handle,
                Some(callback::<F>),
                &mut func as *mut FuncWithState<F> as *mut std::ffi::c_void,
            )
        };
        let res = isl_bool_to_optional_bool(res);
        match res {
            Some(true) => Ok(true),
            Some(false) => Ok(false),
            None => match func.state.into_inner() {
                Ok(()) => Err(self.context_ref().last_error_or_unknown().into()),
                Err(e) => Err(e),
            },
        }
    }
    pub fn foreach_lifted_piece<F>(&self, func: F) -> Result<(), crate::Error>
    where
        F: FnMut(QuasiPolynomial<'a>, Set<'a>) -> Result<(), crate::Error>,
    {
        struct FuncWithState<F> {
            func: F,
            state: Cell<Result<(), crate::Error>>,
        }
        let mut func = FuncWithState {
            func,
            state: Cell::new(Ok(())),
        };
        unsafe extern "C" fn callback<'a, F>(
            set: *mut barvinok_sys::isl_set,
            qpoly: *mut barvinok_sys::isl_qpolynomial,
            user: *mut std::ffi::c_void,
        ) -> barvinok_sys::isl_stat
        where
            F: FnMut(QuasiPolynomial<'a>, Set<'a>) -> Result<(), crate::Error>,
        {
            let data = unsafe { &mut *(user as *mut FuncWithState<F>) };
            let qpoly = QuasiPolynomial {
                handle: NonNull::new(qpoly).unwrap(),
                marker: std::marker::PhantomData,
            };
            let set = Set {
                handle: NonNull::new(set).unwrap(),
                marker: std::marker::PhantomData,
            };
            let state = data.state.replace(Ok(()));
            data.state.set(state.and_then(|_| (data.func)(qpoly, set)));
            if data.state.get_mut().is_ok() {
                barvinok_sys::isl_stat_isl_stat_ok
            } else {
                barvinok_sys::isl_stat_isl_stat_error
            }
        }
        let handle = self.handle.as_ptr();
        let res = unsafe {
            barvinok_sys::isl_pw_qpolynomial_foreach_lifted_piece(
                handle,
                Some(callback::<F>),
                &mut func as *mut FuncWithState<F> as *mut std::ffi::c_void,
            )
        };
        if res == barvinok_sys::isl_stat_isl_stat_ok {
            func.state.into_inner()
        } else {
            match func.state.into_inner() {
                Ok(()) => Err(self.context_ref().last_error_or_unknown().into()),
                Err(e) => Err(e),
            }
        }
    }
    isl_transform!(pw_qpolynomial_coalesce, isl_pw_qpolynomial_coalesce);
    isl_transform!(gist, isl_pw_qpolynomial_gist, [managed] set: Set<'a>);
    isl_transform!(gist_params, isl_pw_qpolynomial_gist_params, [managed] set: Set<'a>);
    isl_transform!(split_periods, isl_pw_qpolynomial_split_periods, [cast(i32)] num_periods: u32);
}

impl<'a> std::ops::Add for PiecewiseQuasiPolynomial<'a> {
    type Output = PiecewiseQuasiPolynomial<'a>;
    fn add(self, other: PiecewiseQuasiPolynomial<'a>) -> Self::Output {
        self.checked_add(other).unwrap()
    }
}

impl<'a> std::ops::Sub for PiecewiseQuasiPolynomial<'a> {
    type Output = PiecewiseQuasiPolynomial<'a>;
    fn sub(self, other: PiecewiseQuasiPolynomial<'a>) -> Self::Output {
        self.checked_sub(other).unwrap()
    }
}

impl<'a> std::ops::Mul for PiecewiseQuasiPolynomial<'a> {
    type Output = PiecewiseQuasiPolynomial<'a>;
    fn mul(self, other: PiecewiseQuasiPolynomial<'a>) -> Self::Output {
        self.checked_mul(other).unwrap()
    }
}

impl<'a> std::ops::Neg for PiecewiseQuasiPolynomial<'a> {
    type Output = PiecewiseQuasiPolynomial<'a>;
    fn neg(self) -> Self::Output {
        self.checked_neg().unwrap()
    }
}

impl<'a> Term<'a> {
    isl_size!(term_dim => dim, [cast(u32)] dim_type: DimType);
    isl_size!(term_get_exp => exponent, [cast(u32)] dim_type: DimType, [trivial] pos: u32);
    isl_project!([into(Value)] coefficient, isl_term_get_coefficient_val);
}

impl<'a> TryFrom<QuasiPolynomial<'a>> for PiecewiseQuasiPolynomial<'a> {
    type Error = crate::Error;
    fn try_from(qpoly: QuasiPolynomial<'a>) -> Result<Self, Self::Error> {
        let ctx = qpoly.context_ref();
        let qpoly = ManuallyDrop::new(qpoly);
        let handle =
            unsafe { barvinok_sys::isl_pw_qpolynomial_from_qpolynomial(qpoly.handle.as_ptr()) };
        NonNull::new(handle)
            .ok_or_else(|| ctx.last_error_or_unknown().into())
            .map(|handle| PiecewiseQuasiPolynomial {
                handle,
                marker: std::marker::PhantomData,
            })
    }
}

impl<'a> TryFrom<Term<'a>> for QuasiPolynomial<'a> {
    type Error = crate::Error;
    fn try_from(term: Term<'a>) -> Result<Self, Self::Error> {
        let ctx = term.context_ref();
        let this = ManuallyDrop::new(term);
        let handle = unsafe { barvinok_sys::isl_qpolynomial_from_term(this.handle.as_ptr()) };
        NonNull::new(handle)
            .ok_or_else(|| ctx.last_error_or_unknown().into())
            .map(|handle| QuasiPolynomial {
                handle,
                marker: std::marker::PhantomData,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_quasi_polynomial_create() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 2).unwrap();
            let qpoly = QuasiPolynomial::zero_on_domain(space).unwrap();
            assert_eq!(qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", qpoly);
        });
    }

    #[test]
    fn test_quasi_polynomial_get_space() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 2).unwrap();
            let qpoly = QuasiPolynomial::one_on_domain(space).unwrap();
            let space2 = qpoly.get_space();
            println!("{:?}", space2);
            let space3 = qpoly.get_domain_space();
            println!("{:?}", space3);
        });
    }

    #[test]
    fn test_quasi_polynomial_get_dim() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 2).unwrap();
            let qpoly = QuasiPolynomial::one_on_domain(space).unwrap();
            let dim = qpoly.get_dim(DimType::Param).unwrap();
            assert_eq!(dim, 1);
            let dim = qpoly.get_dim(DimType::Out).unwrap();
            assert_eq!(dim, 1);
            let dim = qpoly.get_dim(DimType::In).unwrap();
            assert_eq!(dim, 2);
        });
    }

    #[test]
    fn test_quasi_polynomial_new_val_on_domain() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 2).unwrap();
            let value = Value::new_si(ctx, 42);
            let qpoly = QuasiPolynomial::val_on_domain(space, value).unwrap();
            assert_eq!(qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", qpoly);
        });
    }

    #[test]
    fn test_quasi_polynomial_new_var_on_domain() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 2).unwrap();
            let qpoly = QuasiPolynomial::var_on_domain(space.clone(), DimType::Param, 0).unwrap();
            assert_eq!(qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", qpoly);
            let qpoly2 = QuasiPolynomial::var_on_domain(space, DimType::Out, 1).unwrap();
            assert_eq!(qpoly2.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", qpoly2);
        });
    }

    #[test]
    #[should_panic]
    fn test_invalid_var_pos() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 2).unwrap();
            QuasiPolynomial::var_on_domain(space, DimType::Param, 2).unwrap();
        });
    }

    #[test]
    fn test_quasi_polynomial_add() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 2).unwrap();
            let qpoly1 = QuasiPolynomial::one_on_domain(space.clone()).unwrap();
            let qpoly2 = QuasiPolynomial::zero_on_domain(space).unwrap();
            let qpoly3 = qpoly1 + qpoly2;
            assert_eq!(qpoly3.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", qpoly3);
        });
    }

    #[test]
    fn test_quasi_polynomial_sub() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 2).unwrap();
            let qpoly1 = QuasiPolynomial::zero_on_domain(space.clone()).unwrap();
            let qpoly2 = QuasiPolynomial::one_on_domain(space).unwrap();
            let qpoly3 = qpoly1 - qpoly2;
            assert_eq!(qpoly3.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", qpoly3);
            let val = qpoly3.get_constant_val();
            assert!(val.is_ok());
            assert_eq!(val.unwrap().to_f64(), -1.0);
        });
    }

    #[test]
    fn test_pw_qpolynomial_zero() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 2).unwrap();
            let pw_qpoly = PiecewiseQuasiPolynomial::zero(space).unwrap();
            assert_eq!(pw_qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", pw_qpoly);
        });
    }

    #[test]
    fn test_pw_qpolynomial_from_qpolynomial() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 2).unwrap();
            let qpoly = QuasiPolynomial::one_on_domain(space).unwrap();
            let pw_qpoly = PiecewiseQuasiPolynomial::try_from(qpoly).unwrap();
            assert_eq!(pw_qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", pw_qpoly);
        });
    }

    #[test]
    fn test_qpoly_foreach_term() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 2).unwrap();
            let qpoly = QuasiPolynomial::one_on_domain(space).unwrap();
            qpoly
                .foreach_term(|term| {
                    println!("term dim(in): {:?}", term.dim(DimType::In));
                    println!("term dim(out): {:?}", term.dim(DimType::Out));
                    println!("term dim(param): {:?}", term.dim(DimType::Param));
                    println!("term exp(in): {:?}", term.exponent(DimType::In, 0));
                    println!("term exp(out): {:?}", term.exponent(DimType::Out, 0));
                    println!("term exp(param): {:?}", term.exponent(DimType::Param, 0));
                    println!("term coefficient: {:?}", term.coefficient());
                    Ok(())
                })
                .unwrap();
        });
    }
}
