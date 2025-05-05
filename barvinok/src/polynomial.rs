use crate::{
    impl_isl_handle, isl_flag, isl_project, isl_size, isl_transform, stat::{isl_bool_to_optional_bool, isl_size_to_optional_u32}, value::Value
};
use std::{cell::Cell, ptr::NonNull};
use crate::isl_ctor;
use crate::{DimType, space::Space};
use std::mem::ManuallyDrop;

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
        let handle = unsafe { barvinok_sys::isl_pw_qpolynomial_from_qpolynomial(qpoly.handle.as_ptr()) };
        NonNull::new(handle).ok_or_else(|| {
            ctx.last_error_or_unknown().into()
        })
        .map(|handle| {
            PiecewiseQuasiPolynomial {
                handle,
                marker: std::marker::PhantomData,
            }
        })
    }
}

impl<'a> TryFrom<Term<'a>> for QuasiPolynomial<'a> {
    type Error = crate::Error;
    fn try_from(term: Term<'a>) -> Result<Self, Self::Error> {
        let ctx = term.context_ref();
        let this = ManuallyDrop::new(term);
        let handle = unsafe { barvinok_sys::isl_qpolynomial_from_term(this.handle.as_ptr()) };
        NonNull::new(handle).ok_or_else(|| {
            ctx.last_error_or_unknown().into()
        })
        .map(|handle| {
            QuasiPolynomial {
                handle,
                marker: std::marker::PhantomData,
            }
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
            let space = Space::new_set(ctx, 1, 2);
            let qpoly = QuasiPolynomial::zero_on_domain(space).unwrap();
            assert_eq!(qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", qpoly);
        });
    }

    #[test]
    fn test_quasi_polynomial_get_space() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new_set(ctx, 1, 2);
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
            let space = Space::new_set(ctx, 1, 2);
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
            let space = Space::new_set(ctx, 1, 2);
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
            let space = Space::new_set(ctx, 1, 2);
            let qpoly =
                QuasiPolynomial::var_on_domain(space.clone(), DimType::Param, 0).unwrap();
            assert_eq!(qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", qpoly);
            let qpoly2 = QuasiPolynomial::var_on_domain(space, DimType::Out, 1).unwrap();
            assert_eq!(qpoly2.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", qpoly2);
        });
    }

    #[test]
    #[should_panic(expected = "VariablePositionOutOfBounds")]
    fn test_invalid_var_pos() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new_set(ctx, 1, 2);
            QuasiPolynomial::var_on_domain(space, DimType::Param, 2).unwrap();
        });
    }

    #[test]
    fn test_quasi_polynomial_add() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new_set(ctx, 1, 2);
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
            let space = Space::new_set(ctx, 1, 2);
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
            let space = Space::new_set(ctx, 1, 2);
            let pw_qpoly = PiecewiseQuasiPolynomial::zero(space).unwrap();
            assert_eq!(pw_qpoly.context_ref().0.as_ptr(), ctx.0.as_ptr());
            println!("{:?}", pw_qpoly);
        });
    }

    #[test]
    fn test_pw_qpolynomial_from_qpolynomial() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new_set(ctx, 1, 2);
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
            let space = Space::new_set(ctx, 1, 2);
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
