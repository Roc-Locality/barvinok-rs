use std::{cell::Cell, mem::ManuallyDrop, ptr::NonNull};

use crate::{
    DimType,
    constraint::Constraint,
    ident::Ident,
    impl_isl_handle, isl_ctor, isl_flag, isl_project, isl_size, isl_str, isl_transform,
    list::List,
    map::{BasicMap, Map},
    point::Point,
    polynomial::PiecewiseQuasiPolynomial,
    space::Space,
    stat::{isl_bool_to_optional_bool, isl_size_to_optional_u32},
    value::Value,
};

impl_isl_handle!(Set, set);
impl_isl_handle!(BasicSet, basic_set);
macro_rules! set_named_id_methods {
    ($Wrapper:ident, $ctype:ident) => {
        paste::paste! {
            impl<'a> $Wrapper<'a> {
                isl_str!([<$ctype _get_tuple_name>] => [<get_tuple_name>]);
                isl_transform!(set_tuple_name, [<isl_ $ctype _set_tuple_name>], [str] name : &str);
                isl_str!([<$ctype _get_dim_name>] => [<get_dim_name>], [cast(u32)] ty : DimType, [trivial] pos : u32);
                isl_transform!(set_dim_name, [<isl_ $ctype _set_dim_name>], [cast(u32)] ty : DimType, [trivial] pos : u32, [str] name : &str);
                isl_project!([into(Ident)] get_dim_id, [<isl_ $ctype _get_dim_id>], [cast(u32)] ty : DimType, [trivial] pos : u32);
                isl_transform!(set_tuple_id, [<isl_ $ctype _set_tuple_id>], [managed] id : Ident<'a>);
            }
        }
    };
}

set_named_id_methods!(BasicSet, basic_set);
set_named_id_methods!(Set, set);
type ConstraintList<'a> = List<'a, Constraint<'a>>;
#[allow(clippy::should_implement_trait)]
impl<'a> BasicSet<'a> {
    isl_ctor!(universe, isl_basic_set_universe, space : Space<'a>);
    isl_ctor!(empty, isl_basic_set_empty, space : Space<'a>);
    isl_ctor!(nat_universe, isl_basic_set_nat_universe, space : Space<'a>);
    isl_ctor!(positive_orthant, isl_basic_set_positive_orthant, space : Space<'a>);
    isl_size!(basic_set_n_dim => num_dims);
    isl_size!(basic_set_n_param => num_params);
    isl_size!(basic_set_total_dim => total_dims);
    isl_size!(basic_set_dim => get_dims, [cast(u32)] ty : DimType);
    isl_transform!(intersect, isl_basic_set_intersect, [managed] other : BasicSet<'a>);
    isl_transform!(intersect_params, isl_basic_set_intersect_params, [managed] other : BasicSet<'a>);
    isl_transform!(affine_hull, isl_basic_set_affine_hull);
    isl_transform!(sample, isl_basic_set_sample);
    isl_transform!(remove_redundancies, isl_basic_set_remove_redundancies);
    isl_transform!(detect_equalities, isl_basic_set_detect_equalities);
    isl_transform!([into(PiecewiseQuasiPolynomial)] cardinality, isl_basic_set_card);
    isl_transform!(add_constraint, isl_basic_set_add_constraint, [managed] constraint : Constraint<'a>);
    isl_transform!([into(ConstraintList)] get_constraints, isl_basic_set_get_constraint_list);
    isl_transform!(apply, isl_basic_set_apply, [managed] map : BasicMap<'a>);
    isl_transform!(remove_dims, isl_basic_set_remove_dims, [cast(u32)] ty : DimType, [trivial] first : u32, [trivial] num : u32);
    isl_ctor!([ctx] from_str, isl_basic_set_read_from_str, [str] str : &str);
    isl_flag!(basic_set_is_rational => is_rational);
    isl_transform!([into(Set)] lexmin, isl_basic_set_lexmin);
    isl_transform!([into(Set)] lexmax, isl_basic_set_lexmax);
    isl_flag!(basic_set_is_equal => checked_eq, [ref] other : &BasicSet<'a>);
    isl_flag!(basic_set_is_disjoint => disjoint, [ref] other : &BasicSet<'a>);
    isl_transform!([into(Set)] union, isl_basic_set_union, [managed] other : BasicSet<'a>);
    isl_transform!(flat_product, isl_basic_set_flat_product, [managed] other : BasicSet<'a>);
    isl_transform!(checked_neg, isl_basic_set_neg);
    isl_transform!([into(Set)] compute_divs, isl_basic_set_compute_divs);
    isl_transform!(gist, isl_basic_set_gist, [managed] context : BasicSet<'a>);
}
#[allow(clippy::should_implement_trait)]
impl<'a> Set<'a> {
    isl_ctor!(empty, isl_set_empty, space : Space<'a>);
    isl_ctor!(universe, isl_set_universe, space : Space<'a>);
    isl_ctor!(nat_universe, isl_set_nat_universe, space : Space<'a>);
    isl_ctor!(space_universe, isl_space_universe_set, space : Space<'a>);
    isl_transform!(detect_equalities, isl_set_detect_equalities);
    isl_transform!([into(BasicSet)] affine_hull, isl_set_affine_hull);
    isl_transform!([into(BasicSet)] sample, isl_set_sample);
    isl_transform!([into(BasicSet)] convex_hull, isl_set_convex_hull);
    isl_transform!([into(BasicSet)] polyhedral_hull, isl_set_polyhedral_hull);
    isl_transform!([into(BasicSet)] simple_hull, isl_set_simple_hull);
    isl_transform!([into(BasicSet)] unshifted_simple_hull, isl_set_unshifted_simple_hull);
    isl_transform!([into(BasicSet)] plain_unshifted_simple_hull, isl_set_plain_unshifted_simple_hull);
    isl_transform!([into(BasicSet)] bounded_simple_hull, isl_set_bounded_simple_hull);
    isl_transform!(wrapped_reverse, isl_set_wrapped_reverse);
    isl_transform!(disjoint_union, isl_set_union_disjoint, [managed] other : Set<'a>);
    isl_transform!(union, isl_set_union, [managed] other : Set<'a>);
    isl_transform!(product, isl_set_product, [managed] other : Set<'a>);
    isl_transform!(intersect, isl_set_intersect, [managed] other : Set<'a>);
    isl_transform!(intersect_params, isl_set_intersect_params, [managed] other : Set<'a>);
    isl_transform!(intersect_factor_domain, isl_set_intersect_factor_domain, [managed] other : Set<'a>);
    isl_transform!(intersect_factor_range, isl_set_intersect_factor_range, [managed] other : Set<'a>);
    isl_transform!(subtract, isl_set_subtract, [managed] other : Set<'a>);
    isl_transform!(complement, isl_set_complement);
    isl_transform!(move_dims, isl_set_move_dims, [cast(u32)] dst_dim_type : DimType, [trivial] dst_pos : u32, [cast(u32)] src_dim_type : DimType, [trivial] src_pos : u32, [trivial] num : u32);
    isl_size!(set_n_dim => num_dims);
    isl_size!(set_n_param => num_params);
    isl_size!(set_dim => get_dims, [cast(u32)] ty : DimType);
    isl_size!(set_tuple_dim => tuple_dims);
    isl_project!([into(Space)] get_space, isl_set_get_space);
    isl_transform!(reset_space, isl_set_reset_space, [managed] space : Space<'a>);
    isl_flag!(set_has_tuple_name => has_tuple_name);
    isl_flag!(set_has_dim_name => has_dim_name, [cast(u32)] ty : DimType, [trivial] pos : u32);
    isl_flag!(set_has_dim_id => has_dim_id, [cast(u32)] ty : DimType, [trivial] pos : u32);
    isl_flag!(set_has_tuple_id => has_tuple_id);
    isl_project!([into(Ident)] get_tuple_id, isl_set_get_tuple_id);
    isl_transform!(reset_tuple_id, isl_set_reset_tuple_id);
    isl_transform!(reset_user, isl_set_reset_user);
    isl_size!(set_find_dim_by_id => find_dim_by_id, [cast(u32)] ty : DimType, [ref] id : &Ident<'a>);
    isl_size!(set_find_dim_by_name => find_dim_by_name, [cast(u32)] ty : DimType, [str] name : &str);
    isl_transform!(lexmin, isl_set_lexmin);
    isl_transform!(lexmax, isl_set_lexmax);
    isl_flag!(set_is_equal => checked_eq, [ref] other : &Set<'a>);
    isl_flag!(set_is_disjoint => checked_disjoint, [ref] other : &Set<'a>);
    isl_flag!(set_plain_is_equal => plain_is_equal, [ref] other : &Set<'a>);
    pub fn plain_compare(&self, other: &Self) -> std::cmp::Ordering {
        let cmp =
            unsafe { barvinok_sys::isl_set_plain_cmp(self.handle.as_ptr(), other.handle.as_ptr()) };
        cmp.cmp(&0)
    }
    isl_transform!(apply, isl_set_apply, [managed] map : Map<'a>);
    isl_flag!(set_plain_is_empty => plain_is_empty);
    isl_flag!(set_plain_is_universe => plain_is_universe);
    isl_flag!(set_is_params => is_params);
    isl_flag!(set_is_empty => is_empty);
    isl_flag!(set_is_bounded => is_bounded);
    isl_flag!(set_is_singleton => is_singleton);
    isl_flag!(set_is_box => is_box);
    isl_flag!(set_is_subset => subset, [ref] other : &Set<'a>);
    isl_flag!(set_is_strict_subset => strict_subset, [ref] other : &Set<'a>);
    isl_flag!(set_has_equal_space => has_equal_space, [ref] other : &Set<'a>);
    isl_transform!(sum, isl_set_sum, [managed] other : Set<'a>);
    isl_transform!(checked_neg, isl_set_neg);
    isl_transform!(make_disjoint, isl_set_make_disjoint);
    isl_transform!(compute_divs, isl_set_compute_divs);
    isl_flag!(set_dim_is_bounded => dim_is_bounded, [cast(u32)] ty : DimType, [trivial] pos : u32);
    isl_flag!(set_dim_has_lower_bound => dim_has_lower_bound, [cast(u32)] ty : DimType, [trivial] pos : u32);
    isl_flag!(set_dim_has_upper_bound => dim_has_upper_bound, [cast(u32)] ty : DimType, [trivial] pos : u32);
    isl_flag!(set_dim_has_any_lower_bound => dim_has_any_lower_bound, [cast(u32)] ty : DimType, [trivial] pos : u32);
    isl_flag!(set_dim_has_any_upper_bound => dim_has_any_upper_bound, [cast(u32)] ty : DimType, [trivial] pos : u32);
    isl_project!([into(Value)] plain_get_val_if_fixed, isl_set_plain_get_val_if_fixed, [cast(u32)] ty : DimType, [trivial] pos : u32);
    isl_transform!(gist, isl_set_gist, [managed] context : Set<'a>);
    isl_transform!(gist_basic_set, isl_set_gist_basic_set, [managed] context : BasicSet<'a>);
    isl_transform!(gist_params, isl_set_gist_params, [managed] context : Set<'a>);
    isl_transform!(coalesce, isl_set_coalesce);
    isl_size!(set_n_basic_set => num_basic_sets);
    isl_ctor!([ctx] from_str, isl_set_read_from_str, [str] str : &str);
    isl_transform!(add_constraint, isl_set_add_constraint, [managed] constraint : Constraint<'a>);
    isl_transform!([into(PiecewiseQuasiPolynomial)] cardinality, isl_set_card);
    isl_transform!([into(Map)] lex_lt_set, isl_set_lex_lt_set, [managed] set: Set<'a>);
    isl_transform!([into(Map)] lex_le_set, isl_set_lex_le_set, [managed] set: Set<'a>);
    isl_transform!([into(Map)] lex_ge_set, isl_set_lex_ge_set, [managed] set: Set<'a>);
    isl_transform!([into(Map)] lex_gt_set, isl_set_lex_gt_set, [managed] set: Set<'a>);
    isl_transform!(insert_dims, isl_set_insert_dims, [cast(u32)] ty : DimType, [trivial] pos : u32, [trivial] num : u32);
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
        let handle = self.handle.as_ptr();
        let res = unsafe {
            barvinok_sys::isl_set_foreach_point(
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

impl PartialEq for BasicSet<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.checked_eq(other).unwrap_or(false)
    }
}

impl PartialEq for Set<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.checked_eq(other).unwrap_or(false)
    }
}

impl<'a> List<'a, BasicSet<'a>> {
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

impl<'a> List<'a, Set<'a>> {
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
    fn try_from(constraint: Constraint<'a>) -> Result<Self, crate::Error> {
        let ctx = constraint.context_ref();
        let constraint = ManuallyDrop::new(constraint);
        let handle =
            unsafe { barvinok_sys::isl_basic_set_from_constraint(constraint.handle.as_ptr()) };
        let handle = NonNull::new(handle).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(BasicSet {
            handle,
            marker: std::marker::PhantomData,
        })
    }

    type Error = crate::Error;
}

impl<'a> TryFrom<BasicSet<'a>> for Set<'a> {
    fn try_from(basic_set: BasicSet<'a>) -> Result<Self, crate::Error> {
        let ctx = basic_set.context_ref();
        let basic_set = ManuallyDrop::new(basic_set);
        let handle = unsafe { barvinok_sys::isl_set_from_basic_set(basic_set.handle.as_ptr()) };
        let handle = NonNull::new(handle).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(Set {
            handle,
            marker: std::marker::PhantomData,
        })
    }

    type Error = crate::Error;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Context, local_space::LocalSpace};

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
            let mut list = List::new(ctx, 2);
            list.push(basic_set1);
            list.push(basic_set2);
            let intersected_set = list.intersect();
            println!("{:?}", intersected_set);
        });
    }
    #[test]
    fn test_construct_triangular_iteration_space() -> anyhow::Result<()> {
        // for i in 0 .. n
        //     for j in 0 .. i
        //         for k in 0 .. j
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 1, 3).unwrap();
            let local_space = LocalSpace::try_from(space.clone()).unwrap();
            let i_ge_0 = Constraint::new_inequality(local_space.clone())
                .set_coefficient_si(DimType::Out, 0, 1)?
                .set_constant_si(0)?;
            let i_lt_n = Constraint::new_inequality(local_space.clone())
                .set_coefficient_si(DimType::Param, 0, 1)?
                .set_coefficient_si(DimType::Out, 0, -1)?
                .set_constant_si(-1)?;
            let j_ge_0 = Constraint::new_inequality(local_space.clone())
                .set_coefficient_si(DimType::Out, 1, 1)?
                .set_constant_si(0)?;
            let j_lt_i = Constraint::new_inequality(local_space.clone())
                .set_coefficient_si(DimType::Out, 0, 1)?
                .set_coefficient_si(DimType::Out, 1, -1)?
                .set_constant_si(-1)?;
            let k_ge_0 = Constraint::new_inequality(local_space.clone())
                .set_coefficient_si(DimType::Out, 2, 1)?
                .set_constant_si(0)?;
            let k_lt_j = Constraint::new_inequality(local_space.clone())
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
