use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{
    DimType,
    constraint::Constraint,
    impl_isl_handle, nonnull_or_alloc_error,
    polynomial::PiecewiseQuasiPolynomial,
    space::Space,
    stat::{ContextResult, isl_size_to_optional_u32},
};

impl_isl_handle!(Set, set);
impl_isl_handle!(BasicSet, basic_set);

macro_rules! basic_set_constructor {
    ($fn_name:ident, $isl_fn:ident) => {
        paste::paste! {
            pub fn [<new_ $fn_name>](space: Space<'a>) -> Result<Self, crate::Error> {
                if space.get_dim(crate::DimType::In)
                    .context_result(space.context_ref())? != 0
                {
                    return Err(crate::ISLError::Invalid.into());
                }
                let space = ManuallyDrop::new(space);
                let handle = unsafe { barvinok_sys::$isl_fn(space.handle.as_ptr()) };
                let handle = nonnull_or_alloc_error(handle);
                Ok(Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
            }
        }
    };
}

macro_rules! basic_set_unary {
    ($fn_name:ident) => {
        paste::paste! {
            pub fn $fn_name(self) -> Result<Self, crate::Error> {
                let this = ManuallyDrop::new(self);
                let handle = unsafe { barvinok_sys::[<isl_basic_set_ $fn_name>](this.handle.as_ptr()) };
                let handle = nonnull_or_alloc_error(handle);
                Ok(Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
            }
        }
    };
}

macro_rules! basic_set_binary {
    ($fn_name:ident) => {
        paste::paste! {
            pub fn $fn_name(self, other: BasicSet<'a>) -> Result<Self, crate::Error> {
                let this = ManuallyDrop::new(self);
                let other = ManuallyDrop::new(other);
                let handle = unsafe {
                    barvinok_sys::[<isl_basic_set_ $fn_name>](this.handle.as_ptr(), other.handle.as_ptr())
                };
                let handle = nonnull_or_alloc_error(handle);
                Ok(Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
            }
        }
    };
}

impl<'a> BasicSet<'a> {
    basic_set_constructor!(universe, isl_basic_set_universe);
    basic_set_constructor!(empty, isl_basic_set_empty);
    basic_set_constructor!(nat_universe, isl_basic_set_nat_universe);
    basic_set_constructor!(positive_orthant, isl_basic_set_positive_orthant);
    pub fn num_dims(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_n_dim(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn num_params(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_n_param(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn total_dims(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_total_dim(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn get_dims(&self, ty: DimType) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_dim(self.handle.as_ptr(), ty as u32) };
        isl_size_to_optional_u32(num)
    }
    basic_set_binary!(intersect);
    basic_set_binary!(intersect_params);
    basic_set_unary!(affine_hull);
    basic_set_unary!(sample);
    basic_set_unary!(remove_redundancies);
    pub fn cardinality(self) -> Option<PiecewiseQuasiPolynomial<'a>> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_basic_set_card(this.handle.as_ptr()) };
        let handle = NonNull::new(handle)?;
        Some(PiecewiseQuasiPolynomial {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn add_constraint(self, constraint: Constraint<'a>) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let constraint = ManuallyDrop::new(constraint);
        let handle = unsafe {
            barvinok_sys::isl_basic_set_add_constraint(
                this.handle.as_ptr(),
                constraint.handle.as_ptr(),
            )
        };
        NonNull::new(handle).map(|handle| BasicSet {
            handle,
            marker: std::marker::PhantomData,
        })
    }
}

impl<'a> From<Constraint<'a>> for BasicSet<'a> {
    fn from(constraint: Constraint<'a>) -> Self {
        let constraint = ManuallyDrop::new(constraint);
        let handle =
            unsafe { barvinok_sys::isl_basic_set_from_constraint(constraint.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        BasicSet {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Context, local_space::LocalSpace};

    #[test]
    fn test_basic_set_creation() {
        let ctx = Context::new();
        let space = Space::new(&ctx, 0, 0, 3);
        let basic_set = BasicSet::new_universe(space.clone()).unwrap();
        println!("{:?}", basic_set);
        let basic_set = BasicSet::new_empty(space.clone()).unwrap();
        println!("{:?}", basic_set);
        let basic_set = BasicSet::new_nat_universe(space.clone()).unwrap();
        println!("{:?}", basic_set);
        let basic_set = BasicSet::new_positive_orthant(space.clone()).unwrap();
        println!("{:?}", basic_set);
    }

    #[test]
    fn test_basic_set_bin_ops() {
        let ctx = Context::new();
        let space = Space::new(&ctx, 0, 0, 3);
        let basic_set1 = BasicSet::new_universe(space.clone()).unwrap();
        let basic_set2 = BasicSet::new_empty(space.clone()).unwrap();
        let basic_set3 = basic_set1.intersect(basic_set2).unwrap();
        println!("{:?}", basic_set3);
    }

    #[test]
    fn test_basic_set_unary_ops() {
        let ctx = Context::new();
        let space = Space::new(&ctx, 0, 0, 3);
        let basic_set = BasicSet::new_positive_orthant(space.clone()).unwrap();
        let basic_set = basic_set.affine_hull().unwrap();
        println!("{:?}", basic_set);
    }
    #[test]
    fn test_basic_set_cardinality() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 1, 4);
        let basic_set = BasicSet::new_universe(space.clone()).unwrap();
        let card = basic_set.cardinality().unwrap();
        println!("{:?}", card);
    }
    #[test]
    fn test_interval_product_space() {
        let ctx = Context::new();
        let space = Space::new_set(&ctx, 3, 3);
        let local_space = LocalSpace::from(space.clone());
        let mut set = BasicSet::new_universe(space.clone()).unwrap();
        for i in 0..3 {
            {
                let mut i_ge_0 = Constraint::new_inequality(local_space.clone());
                i_ge_0 = i_ge_0.set_coefficient_si(DimType::Out, i, 1).unwrap();
                set = set.add_constraint(i_ge_0).unwrap();
                println!("{:?}", set);
            }
            {
                let mut i_lt_p = Constraint::new_inequality(local_space.clone());
                i_lt_p = i_lt_p.set_coefficient_si(DimType::Param, i, 1).unwrap();
                i_lt_p = i_lt_p.set_coefficient_si(DimType::Out, i, -1).unwrap();
                i_lt_p = i_lt_p.set_constant_si(-1).unwrap();
                set = set.add_constraint(i_lt_p).unwrap();
                println!("{:?}", set);
            }
        }
        let card = set.cardinality().unwrap();
        println!("{:?}", card);
    }
}
