use crate::aff::Affine;
use crate::set::Set;
use crate::stat::isl_bool_to_optional_bool;
use crate::value::Value;
use crate::{DimType, impl_isl_handle, isl_project, isl_size};
use std::{cell::Cell, ptr::NonNull};

impl_isl_handle!([printer] QuasiPolynomial, qpolynomial);
impl_isl_handle!([noprint nodump] Term, term);
impl_isl_handle!(PiecewiseQuasiPolynomial, pw_qpolynomial);

include!(concat!(env!("OUT_DIR"), "/generated/qpolynomial.rs"));
include!(concat!(env!("OUT_DIR"), "/generated/pw_qpolynomial.rs"));

impl<'a> QuasiPolynomial<'a> {
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
    isl_size!(isl_term_dim => dim, [cast(u32)] dim_type: DimType);
    isl_size!(isl_term_get_exp => exponent, [cast(u32)] dim_type: DimType, [trivial] pos: u32);
    isl_project!([into(Affine)] get_div, isl_term_get_div, [trivial] pos: u32);
    isl_project!([into(Value)] coefficient, isl_term_get_coefficient_val);
}

impl<'a> TryFrom<QuasiPolynomial<'a>> for PiecewiseQuasiPolynomial<'a> {
    type Error = crate::Error;
    fn try_from(qpoly: QuasiPolynomial<'a>) -> Result<Self, Self::Error> {
        Self::from_qpolynomial(qpoly)
    }
}

impl<'a> TryFrom<Term<'a>> for QuasiPolynomial<'a> {
    type Error = crate::Error;
    fn try_from(term: Term<'a>) -> Result<Self, Self::Error> {
        Self::from_term(term)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;
    use crate::space::Space;

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
            let value = Value::int_from_si(ctx, 42).unwrap();
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
