use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::aff::Affine;
use crate::ident::Ident;
use crate::local_space::LocalSpace;
use crate::set::Set;
use crate::space::Space;
use crate::stat::isl_bool_to_optional_bool;
use crate::value::Value;
use crate::{DimType, constraint::Constraint, impl_isl_handle, stat::isl_size_to_optional_u32};
use crate::{isl_ctor, isl_flag, isl_project, isl_size, isl_transform};

impl_isl_handle!(Map, map);
impl_isl_handle!(BasicMap, basic_map);

impl<'a> TryFrom<Constraint<'a>> for BasicMap<'a> {
    fn try_from(constraint: Constraint<'a>) -> Result<Self, Self::Error> {
        let ctx = constraint.context_ref();
        let constraint = ManuallyDrop::new(constraint);
        let handle =
            unsafe { barvinok_sys::isl_basic_map_from_constraint(constraint.handle.as_ptr()) };
        let handle = NonNull::new(handle).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(BasicMap {
            handle,
            marker: std::marker::PhantomData,
        })
    }

    type Error = crate::Error;
}

impl<'a> BasicMap<'a> {
    isl_size!(basic_map_total_dim => total_dim);
    isl_size!(basic_map_dim => dim, [cast(u32)] dim_type: DimType);
    isl_project!([into(Space)] get_space, isl_basic_map_get_space);
    isl_project!([into(LocalSpace)] get_local_space, isl_basic_map_get_local_space);
    isl_project!([into(Affine)] get_div, isl_basic_map_get_div, [cast(i32)] pos: u32);
}

#[allow(clippy::should_implement_trait)]
impl<'a> Map<'a> {
    isl_size!(map_domain_tuple_dim => domain_tuple_dim);
    isl_size!(map_range_tuple_dim => range_tuple_dim);
    isl_size!(map_dim => dim, [cast(u32)] dim_type: DimType);
    isl_project!([into(Space)] get_space, isl_map_get_space);
    isl_ctor!(lex_lt, isl_map_lex_lt, space: Space<'a>);
    isl_ctor!(lex_le, isl_map_lex_le, space: Space<'a>);
    isl_ctor!(lex_ge, isl_map_lex_ge, space: Space<'a>);
    isl_ctor!(lex_gt, isl_map_lex_gt, space: Space<'a>);
    isl_ctor!(lex_lt_first, isl_map_lex_lt_first, space: Space<'a>, [trivial] first: u32);
    isl_ctor!(lex_le_first, isl_map_lex_le_first, space: Space<'a>, [trivial] first: u32);
    isl_ctor!(lex_ge_first, isl_map_lex_ge_first, space: Space<'a>, [trivial] first: u32);
    isl_ctor!(lex_gt_first, isl_map_lex_gt_first, space: Space<'a>, [trivial] first: u32);
    isl_ctor!(universe, isl_map_universe, space: Space<'a>);
    isl_ctor!(empty, isl_map_empty, space: Space<'a>);
    isl_ctor!(identity, isl_map_identity, space: Space<'a>);
    isl_transform!(reverse, isl_map_reverse);
    isl_transform!(domain_reverse, isl_map_domain_reverse);
    isl_transform!(sum, isl_map_sum, [managed] map: Map<'a>);
    isl_transform!(neg, isl_map_neg);
    isl_transform!(floor_div, isl_map_floordiv_val, [managed] aff: Value<'a>);
    isl_flag!(map_is_disjoint => disjoint, [ref] other: &Self);
    isl_flag!(map_is_equal => equal, [ref] other: &Self);
    isl_transform!(lexmin, isl_map_lexmin);
    isl_transform!(lexmax, isl_map_lexmax);
    isl_transform!(range_reverse, isl_map_range_reverse);
    isl_transform!(union, isl_map_union, [managed] map: Map<'a>);
    isl_transform!(disjoint_union, isl_map_union_disjoint, [managed] map: Map<'a>);
    isl_transform!(intersect_domain, isl_map_intersect_domain, [managed] set: Set<'a>);
    isl_transform!(intersect_range, isl_map_intersect_range, [managed] set: Set<'a>);
    isl_transform!(intersect_domain_factor_domain, isl_map_intersect_domain_factor_domain, [managed] map: Map<'a>);
    isl_transform!(intersect_range_factor_range, isl_map_intersect_range_factor_range, [managed] map: Map<'a>);
    isl_transform!(intersect_range_factor_domain, isl_map_intersect_range_factor_domain, [managed] map: Map<'a>);
    isl_transform!(intersect_domain_factor_range, isl_map_intersect_domain_factor_range, [managed] map: Map<'a>);
    isl_transform!(intersect_domain_wrapped_domain, isl_map_intersect_domain_wrapped_domain, [managed] set: Set<'a>);
    isl_transform!(intersect_range_wrapped_range, isl_map_intersect_range_wrapped_domain, [managed] set: Set<'a>);
    isl_transform!(apply_domain, isl_map_apply_domain, [managed] map: Map<'a>);
    isl_transform!(apply_range, isl_map_apply_range, [managed] map: Map<'a>);
    isl_transform!(product, isl_map_product, [managed] map: Map<'a>);
    isl_transform!(domain_product, isl_map_domain_product, [managed] map: Map<'a>);
    isl_transform!(range_product, isl_map_range_product, [managed] map: Map<'a>);
    isl_transform!(flat_product, isl_map_flat_product, [managed] map: Map<'a>);
    isl_transform!(flat_domain_product, isl_map_flat_domain_product, [managed] map: Map<'a>);
    isl_transform!(flat_range_product, isl_map_flat_range_product, [managed] map: Map<'a>);
    isl_flag!(map_domain_is_wrapping => domain_is_wrapping);
    isl_flag!(map_range_is_wrapping => range_is_wrapping);
    isl_flag!(map_is_product => is_product);
    isl_transform!(factor_domain, isl_map_factor_domain);
    isl_transform!(factor_range, isl_map_factor_range);
    isl_transform!(domain_factor_domain, isl_map_domain_factor_domain);
    isl_transform!(range_factor_range, isl_map_range_factor_range);
    isl_transform!(domain_factor_range, isl_map_domain_factor_range);
    isl_transform!(range_factor_domain, isl_map_range_factor_domain);
    isl_transform!(intersect, isl_map_intersect, [managed] map: Map<'a>);
    isl_transform!(intersect_params, isl_map_intersect_params, [managed] set: Set<'a>);
    isl_transform!(subtract, isl_map_subtract, [managed] map: Map<'a>);
    isl_transform!(subtract_domain, isl_map_subtract_domain, [managed] set: Set<'a>);
    isl_transform!(subtract_range, isl_map_subtract_range, [managed] set: Set<'a>);
    isl_transform!(complement, isl_map_complement);
    isl_transform!(fix_input_si, isl_map_fix_input_si, [trivial] input: u32, [trivial] value: i32);
    isl_transform!(fix_si, isl_map_fix_si, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] value: i32);
    isl_transform!(fix_val, isl_map_fix_val, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [managed] value: Value<'a>);
    isl_transform!(lower_bound_si, isl_map_lower_bound_si, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] value: i32);
    isl_transform!(lower_bound_val, isl_map_lower_bound_val, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [managed] value: Value<'a>);
    isl_transform!(upper_bound_si, isl_map_upper_bound_si, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] value: i32);
    isl_transform!(upper_bound_val, isl_map_upper_bound_val, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [managed] value: Value<'a>);
    isl_transform!([into(Set)] deltas, isl_map_deltas);
    isl_transform!(deltas_map, isl_map_deltas_map);
    isl_transform!(detect_equalities, isl_map_detect_equalities);
    isl_transform!(add_dims, isl_map_add_dims, [cast(u32)] dim_type: DimType, [trivial] num: u32);
    isl_transform!(insert_dims, isl_map_insert_dims, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    isl_transform!(move_dims, isl_map_move_dims, [cast(u32)] dim_type: DimType, [cast(u32)] dst_type: DimType, [trivial] dst_pos: u32, [trivial] src_pos: u32, [trivial] num: u32);
    isl_transform!(project_out_param_id, isl_map_project_out_param_id, [managed] ident: Ident<'a>);
    isl_transform!(project_out, isl_map_project_out, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    isl_transform!(project_out_all_params, isl_map_project_out_all_params);
    isl_transform!(remove_unknown_divs, isl_map_remove_unknown_divs);
    isl_transform!(remove_divs, isl_map_remove_divs);
    isl_transform!(eliminate, isl_map_eliminate, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    isl_transform!(remove_dims, isl_map_remove_dims, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    isl_transform!(remove_divs_involving_dims, isl_map_remove_divs_involving_dims, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    isl_transform!(remove_inputs, isl_map_remove_inputs, [trivial] pos: u32, [trivial] num: u32);
}

#[cfg(test)]
mod tests {
    use crate::{
        Context, DimType,
        constraint::Constraint,
        local_space::LocalSpace,
        map::{BasicMap, Map},
        space::Space,
    };

    #[test]
    fn test_from_constraints() {
        let context = Context::new();
        context.scope(|context| {
            let space = Space::new(context, 1, 2, 2);
            let local_space = LocalSpace::from(space);
            let mut constraint = Constraint::new_inequality(local_space);
            constraint = constraint.set_constant_si(5).unwrap();
            constraint = constraint.set_coefficient_si(DimType::Param, 0, 3).unwrap();
            let basic_set = BasicMap::try_from(constraint).unwrap();
            println!("Basic Map: {:?}", basic_set);
        });
    }

    #[test]
    fn test_lex_lt_on_space() -> anyhow::Result<()> {
        let context = Context::new();
        context.scope(|context| {
            let space = Space::new_set(context, 2, 2);
            let basic_map = Map::lex_lt(space)?;
            println!("Lexicographically less than map: {:?}", basic_map);
            Ok(())
        })
    }
}
