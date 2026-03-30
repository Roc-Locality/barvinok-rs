use std::mem::ManuallyDrop;

#[allow(unused_imports)]
use crate::DimType;
use crate::{aff::Affine, impl_isl_handle, nonnull_or_alloc_error};

impl_isl_handle!([printer] Constraint, constraint);

include!(concat!(env!("OUT_DIR"), "/generated/constraint.rs"));

impl<'a> Constraint<'a> {
    pub fn new_inequality_from_affine(affine: Affine<'a>) -> Self {
        let affine = ManuallyDrop::new(affine);
        let handle = unsafe { barvinok_sys::isl_inequality_from_aff(affine.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }

    pub fn new_equality_from_affine(affine: Affine<'a>) -> Self {
        let affine = ManuallyDrop::new(affine);
        let handle = unsafe { barvinok_sys::isl_equality_from_aff(affine.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }

    pub fn plain_cmp(&self, other: &Self) -> std::cmp::Ordering {
        let res = unsafe {
            barvinok_sys::isl_constraint_plain_cmp(self.handle.as_ptr(), other.handle.as_ptr())
        };
        res.cmp(&0)
    }

    pub fn cmp_last_nonzero(&self, other: &Self) -> std::cmp::Ordering {
        let res = unsafe {
            barvinok_sys::isl_constraint_cmp_last_non_zero(
                self.handle.as_ptr(),
                other.handle.as_ptr(),
            )
        };
        res.cmp(&0)
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::set::BasicSet;

    use super::*;
    use crate::{DimType, local_space::LocalSpace, space::Space};

    #[test]
    fn test_new_equality() {
        let context = Context::new();
        context.scope(|context| {
            let space = Space::new(context, 1, 2, 2).unwrap();
            let local_space = LocalSpace::try_from(space).unwrap();
            let constraint = Constraint::new_equality(local_space).unwrap();
            println!("Constraint: {:?}", constraint);
        });
    }

    #[test]
    fn test_new_inequality() {
        let context = Context::new();
        context.scope(|context| {
            let space = Space::new(context, 1, 2, 2).unwrap();
            let local_space = LocalSpace::try_from(space).unwrap();
            let constraint = Constraint::new_inequality(local_space).unwrap();
            println!("Constraint: {:?}", constraint);
        });
    }

    #[test]
    fn test_inequality_constant_and_coeff() {
        let context = Context::new();
        context.scope(|context| {
            let space = Space::new(context, 1, 2, 2).unwrap();
            let local_space = LocalSpace::try_from(space).unwrap();
            let constraint = Constraint::new_inequality(local_space).unwrap();
            let constant_val = constraint.get_constant().unwrap();
            println!("Constant Value: {:?}", constant_val);
            let coeff_val = constraint.get_coefficient(DimType::Param, 0).unwrap();
            println!("Coefficient Value: {:?}", coeff_val);
        });
    }

    #[test]
    fn test_set_constant_and_coeff() {
        let context = Context::new();
        context.scope(|context| {
            let space = Space::new(context, 1, 2, 2).unwrap();
            let local_space = LocalSpace::try_from(space).unwrap();
            let mut constraint = Constraint::new_inequality(local_space).unwrap();
            constraint = constraint.set_constant_si(5).unwrap();
            constraint = constraint.set_coefficient_si(DimType::Param, 0, 3).unwrap();
            println!("Updated Constraint: {:?}", constraint);
        });
    }

    #[test]
    fn test_negate() {
        let context = Context::new();
        context.scope(|context| {
            let space = Space::new(context, 1, 2, 2).unwrap();
            let local_space = LocalSpace::try_from(space).unwrap();
            let mut constraint = Constraint::new_inequality(local_space).unwrap();
            constraint = constraint.set_constant_si(5).unwrap();
            constraint = constraint.set_coefficient_si(DimType::Param, 0, 3).unwrap();
            let negated_constraint = constraint.negate().unwrap();
            println!("Negated Constraint: {:?}", negated_constraint);
        });
    }

    #[test]
    fn test_get_affine() {
        let context = Context::new();
        context.scope(|context| {
            let space = Space::set(context, 1, 2).unwrap();
            let local_space = LocalSpace::try_from(space).unwrap();
            let mut constraint = Constraint::new_inequality(local_space).unwrap();
            constraint = constraint.set_constant_si(5).unwrap();
            constraint = constraint.set_coefficient_si(DimType::Param, 0, 3).unwrap();
            let affine = constraint.get_affine().unwrap();
            println!("Affine: {:?}", affine);
        });
    }

    #[test]
    fn test_into_basic_set() {
        let context = Context::new();
        context.scope(|context| {
            let space = Space::set(context, 1, 2).unwrap();
            let local_space = LocalSpace::try_from(space).unwrap();
            let mut constraint = Constraint::new_inequality(local_space).unwrap();
            constraint = constraint.set_constant_si(5).unwrap();
            constraint = constraint.set_coefficient_si(DimType::Param, 0, 3).unwrap();
            let basic_set = BasicSet::try_from(constraint).unwrap();
            println!("Basic Set: {:?}", basic_set);
        });
    }
}
