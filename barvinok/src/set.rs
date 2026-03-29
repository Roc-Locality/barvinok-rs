use std::{cell::Cell, mem::ManuallyDrop, ptr::NonNull};

use crate::{
    DimType,
    constraint::Constraint,
    impl_isl_handle,
    list::{BasicSetList, ConstraintList, SetList},
    point::Point,
};

impl_isl_handle!(Set, set);
impl_isl_handle!(BasicSet, basic_set);

include!(concat!(env!("OUT_DIR"), "/generated/basic_set.rs"));
include!(concat!(env!("OUT_DIR"), "/generated/set.rs"));

impl<'a> BasicSet<'a> {
    crate::isl_transform!(
        [into(ConstraintList<'a>)] get_constraints,
        isl_basic_set_get_constraint_list
    );
}

impl<'a> Set<'a> {
    pub fn plain_compare(&self, other: &Self) -> std::cmp::Ordering {
        let cmp =
            unsafe { barvinok_sys::isl_set_plain_cmp(self.handle.as_ptr(), other.handle.as_ptr()) };
        cmp.cmp(&0)
    }

    pub fn foreach_point<F>(&self, func: F) -> Result<(), crate::Error>
    where
        F: FnMut(Point<'a>) -> Result<(), crate::Error>,
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
            point: *mut barvinok_sys::isl_point,
            user: *mut std::ffi::c_void,
        ) -> barvinok_sys::isl_stat
        where
            F: FnMut(Point<'a>) -> Result<(), crate::Error>,
        {
            let data = unsafe { &mut *(user as *mut FuncWithState<F>) };
            let point = Point {
                handle: NonNull::new(point).unwrap(),
                marker: std::marker::PhantomData,
            };
            let state = data.state.replace(Ok(()));
            data.state.set(state.and_then(|_| (data.func)(point)));
            if data.state.get_mut().is_ok() {
                barvinok_sys::isl_stat_isl_stat_ok
            } else {
                barvinok_sys::isl_stat_isl_stat_error
            }
        }

        let res = unsafe {
            barvinok_sys::isl_set_foreach_point(
                self.handle.as_ptr(),
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

impl PartialEq for BasicSet<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other).unwrap_or(false)
    }
}

impl PartialEq for Set<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other).unwrap_or(false)
    }
}

impl<'a> BasicSetList<'a> {
    pub fn intersect(self) -> BasicSet<'a> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_basic_set_list_intersect(this.handle.as_ptr()) };
        let handle = NonNull::new(handle).unwrap();
        BasicSet {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a> SetList<'a> {
    pub fn union(self) -> Set<'a> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_set_list_union(this.handle.as_ptr()) };
        let handle = NonNull::new(handle).unwrap();
        Set {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a> TryFrom<Constraint<'a>> for BasicSet<'a> {
    type Error = crate::Error;

    fn try_from(constraint: Constraint<'a>) -> Result<Self, Self::Error> {
        Self::from_constraint(constraint)
    }
}

impl<'a> TryFrom<BasicSet<'a>> for Set<'a> {
    type Error = crate::Error;

    fn try_from(basic_set: BasicSet<'a>) -> Result<Self, Self::Error> {
        Self::from_basic_set(basic_set)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Context, local_space::LocalSpace, space::Space};

    #[test]
    fn test_basic_set_creation() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 0, 0, 3).unwrap();
            let basic_set = BasicSet::universe(space.clone()).unwrap();
            println!("{:?}", basic_set);
            let basic_set = BasicSet::empty(space.clone()).unwrap();
            println!("{:?}", basic_set);
            let basic_set = BasicSet::nat_universe(space.clone()).unwrap();
            println!("{:?}", basic_set);
            let basic_set = BasicSet::positive_orthant(space.clone()).unwrap();
            println!("{:?}", basic_set);
        });
    }

    #[test]
    fn test_basic_set_bin_ops() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 0, 0, 3).unwrap();
            let basic_set1 = BasicSet::universe(space.clone()).unwrap();
            let basic_set2 = BasicSet::empty(space.clone()).unwrap();
            let basic_set3 = basic_set1.intersect(basic_set2).unwrap();
            println!("{:?}", basic_set3);
        });
    }

    #[test]
    fn test_basic_set_unary_ops() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 0, 0, 3).unwrap();
            let basic_set = BasicSet::positive_orthant(space.clone()).unwrap();
            let basic_set = basic_set.affine_hull().unwrap();
            println!("{:?}", basic_set);
        });
    }

    #[test]
    fn test_basic_set_cardinality() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 4).unwrap();
            let basic_set = BasicSet::universe(space.clone()).unwrap();
            let card = basic_set.cardinality().unwrap();
            println!("{:?}", card);
        });
    }

    #[test]
    fn test_interval_product_space() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 3, 3).unwrap();
            let local_space = LocalSpace::try_from(space.clone()).unwrap();
            let mut set = BasicSet::universe(space.clone()).unwrap();
            for i in 0..3 {
                {
                    let i_ge_0 = Constraint::new_inequality(local_space.clone())
                        .unwrap()
                        .set_coefficient_si(DimType::Out, i, 1)
                        .unwrap();
                    set = set.add_constraint(i_ge_0).unwrap();
                    println!("{:?}", set);
                }
                {
                    let i_lt_p = Constraint::new_inequality(local_space.clone())
                        .unwrap()
                        .set_coefficient_si(DimType::Param, i, 1)
                        .unwrap()
                        .set_coefficient_si(DimType::Out, i, -1)
                        .unwrap()
                        .set_constant_si(-1)
                        .unwrap();
                    set = set.add_constraint(i_lt_p).unwrap();
                    println!("{:?}", set);
                }
            }
            let card = set.clone().cardinality().unwrap();
            println!("{:?}", card);
            println!("constraints:");
            let list = set.get_constraints().unwrap();
            for i in list.iter() {
                println!("{:?}", i);
            }
        });
    }

    #[test]
    fn test_basic_set_from_str() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let basic_set =
                BasicSet::from_str(ctx, "[p0, p1] -> { [i0, i1] : 5i0 + 6i1 >= p1 - p0 }").unwrap();
            println!("{:?}", basic_set);
        });
    }

    #[test]
    fn test_basic_set_list_intersect() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 5).unwrap();
            let basic_set1 = BasicSet::universe(space.clone()).unwrap();
            let basic_set2 = BasicSet::empty(space.clone()).unwrap();
            let mut list = BasicSetList::new(ctx, 2);
            list.push(basic_set1);
            list.push(basic_set2);
            let intersected_set = list.intersect();
            println!("{:?}", intersected_set);
        });
    }

    #[test]
    fn test_dim_removal() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let set = Set::from_str(ctx, "[R] -> { [i] : 0 <= i <= 10 and R >= 5 }").unwrap();
            let set = set.remove_dims(DimType::Param, 0, 1).unwrap();
            println!("{:?}", set);
        });
    }

    #[test]
    fn test_construct_triangular_iteration_space() -> anyhow::Result<()> {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 3).unwrap();
            let local_space = LocalSpace::try_from(space.clone()).unwrap();
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
            let set = Set::universe(space.clone())?
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
            println!("iteration space {:?}", set);
            let card = set.cardinality()?;
            println!("cardinality {:?}", card);
            Ok(())
        })
    }
}
