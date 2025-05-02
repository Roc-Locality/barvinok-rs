use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::aff::Affine;
use crate::ident::Ident;
use crate::local_space::LocalSpace;
use crate::set::Set;
use crate::space::Space;
use crate::stat::isl_bool_to_optional_bool;
use crate::value::Value;
use crate::{DimType, constraint::Constraint, impl_isl_handle, stat::isl_size_to_optional_u32};

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

macro_rules! map_get_managed {
    ([keep] $func_name:ident, $ctype:ident, $res_ty:ty) => {
        paste::paste! {
            pub fn $func_name(&self) -> Result<$res_ty<'a>, $crate::Error> {
                let handle = unsafe { barvinok_sys::[<isl_ $ctype _ $func_name>](self.handle.as_ptr()) };
                NonNull::new(handle)
                    .ok_or_else(|| self.context_ref().last_error_or_unknown().into())
                    .map(|handle| {
                        $res_ty {
                            handle,
                            marker: std::marker::PhantomData,
                        }
                    })
            }
        }
    };
    ([take] $func_name:ident, $ctype:ident, $res_ty:ty) => {
        paste::paste! {
            pub fn $func_name(self) -> Result<$res_ty<'a>, $crate::Error> {
                let ctx = self.context_ref();
                let this = ManuallyDrop::new(self);
                let handle = unsafe { barvinok_sys::[<isl_ $ctype _ $func_name>](this.handle.as_ptr()) };
                NonNull::new(handle)
                    .ok_or_else(|| ctx.last_error_or_unknown().into())
                    .map(|handle| {
                        $res_ty {
                            handle,
                            marker: std::marker::PhantomData,
                        }
                    })
            }
        }
    };
}

impl<'a> BasicMap<'a> {
    pub fn total_dims(&self) -> Option<u32> {
        let handle = unsafe { barvinok_sys::isl_basic_map_total_dim(self.handle.as_ptr()) };
        isl_size_to_optional_u32(handle)
    }
    pub fn dim(&self, dim_type: DimType) -> Option<u32> {
        let handle =
            unsafe { barvinok_sys::isl_basic_map_dim(self.handle.as_ptr(), dim_type as u32) };
        isl_size_to_optional_u32(handle)
    }
    map_get_managed!([keep] get_space, basic_map, Space);
    map_get_managed!([keep] get_local_space, basic_map, LocalSpace);
    pub fn get_div(&self, pos: u32) -> Result<Affine<'a>, crate::Error> {
        let handle =
            unsafe { barvinok_sys::isl_basic_map_get_div(self.handle.as_ptr(), pos as i32) };
        NonNull::new(handle)
            .ok_or_else(|| self.context_ref().last_error_or_unknown().into())
            .map(|handle| Affine {
                handle,
                marker: std::marker::PhantomData,
            })
    }
}

macro_rules! map_ctor {
    (@get_access [trivial] $val:ident) => {
        $val
    };
    (@get_access [managed] $val:ident) => {
        $val.handle.as_ptr()
    };
    (@take [trivial] $val:ident) => {
        $val
    };
    (@take [managed] $val:ident) => {
        ManuallyDrop::new($val)
    };
    ($func:ident, $sys_fn:ident,
     $first_name:ident : $first_ty:ty
     $(, [$kind:ident] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            $first_name: $first_ty
            $(, $name: $ty )*
        ) -> Result<Self, crate::Error> {
            // pull the ContextRef from the first argument
            let ctx = $first_name.context_ref();
            // consume each arg into ManuallyDrop
            let $first_name = std::mem::ManuallyDrop::new($first_name);
            $(
                let $name = map_ctor!(@take [$kind] $name);
            )*

            // call the raw C function
            let raw = unsafe {
                barvinok_sys::$sys_fn(
                    $first_name.handle.as_ptr()
                    $(, map_ctor!(@get_access [$kind] $name) )*
                )
            };

            // wrap in NonNull, use saved `ctx` on error
            NonNull::new(raw)
                .ok_or_else(|| ctx.last_error_or_unknown().into())
                .map(|handle| Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
        }
    };
}

macro_rules! map_transform {
    (@get_access [trivial] $val:ident) => {
        $val as _
    };
    (@get_access [managed] $val:ident) => {
        $val.handle.as_ptr()
    };
    (@take [trivial] $val:ident) => {
        $val
    };
    (@take [managed] $val:ident) => {
        ManuallyDrop::new($val)
    };
    ($func:ident, $sys_fn:ident
     $(, [$kind:ident] $name:ident : $ty:ty )* $(,)? ) => {
        pub fn $func(
            self: Self
            $(, $name: $ty )*
        ) -> Result<Self, crate::Error> {
            // pull the ContextRef from the first argument
            let ctx = self.context_ref();
            // consume each arg into ManuallyDrop
            let this = std::mem::ManuallyDrop::new(self);
            $(
                let $name = map_transform!(@take [$kind] $name);
            )*

            // call the raw C function
            let raw = unsafe {
                barvinok_sys::$sys_fn(
                    this.handle.as_ptr()
                    $(, map_transform!(@get_access [$kind] $name) )*
                )
            };

            // wrap in NonNull, use saved `ctx` on error
            NonNull::new(raw)
                .ok_or_else(|| ctx.last_error_or_unknown().into())
                .map(|handle| Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
        }
    };
}

macro_rules! map_flag {
    ($isl_func:ident => $fn_name:ident) => {
        paste::paste! {
            pub fn $fn_name(&self) -> Option<bool> {
                let flag = unsafe { barvinok_sys::[<isl_ $isl_func>](self.handle.as_ptr()) };
                isl_bool_to_optional_bool(flag)
            }
        }
    };
}

#[allow(clippy::should_implement_trait)]
impl<'a> Map<'a> {
    pub fn domain_tuple_dim(&self) -> Option<u32> {
        let handle = unsafe { barvinok_sys::isl_map_domain_tuple_dim(self.handle.as_ptr()) };
        isl_size_to_optional_u32(handle)
    }
    pub fn range_tuple_dim(&self) -> Option<u32> {
        let handle = unsafe { barvinok_sys::isl_map_range_tuple_dim(self.handle.as_ptr()) };
        isl_size_to_optional_u32(handle)
    }
    pub fn dim(&self, dim_type: DimType) -> Option<u32> {
        let handle = unsafe { barvinok_sys::isl_map_dim(self.handle.as_ptr(), dim_type as u32) };
        isl_size_to_optional_u32(handle)
    }
    map_get_managed!([keep] get_space, map, Space);
    map_ctor!(lex_lt, isl_map_lex_lt, space: Space<'a>);
    map_ctor!(lex_le, isl_map_lex_le, space: Space<'a>);
    map_ctor!(lex_ge, isl_map_lex_ge, space: Space<'a>);
    map_ctor!(lex_gt, isl_map_lex_gt, space: Space<'a>);
    map_ctor!(lex_lt_first, isl_map_lex_lt_first, space: Space<'a>, [trivial] first: u32);
    map_ctor!(lex_le_first, isl_map_lex_le_first, space: Space<'a>, [trivial] first: u32);
    map_ctor!(lex_ge_first, isl_map_lex_ge_first, space: Space<'a>, [trivial] first: u32);
    map_ctor!(lex_gt_first, isl_map_lex_gt_first, space: Space<'a>, [trivial] first: u32);
    map_ctor!(universe, isl_map_universe, space: Space<'a>);
    map_ctor!(empty, isl_map_empty, space: Space<'a>);
    map_ctor!(identity, isl_map_identity, space: Space<'a>);
    map_transform!(reverse, isl_map_reverse);
    map_transform!(domain_reverse, isl_map_domain_reverse);
    map_transform!(sum, isl_map_sum, [managed] map: Map<'a>);
    map_transform!(neg, isl_map_neg);
    map_transform!(floor_div, isl_map_floordiv_val, [managed] aff: Value<'a>);
    pub fn equal(&self, other: &Self) -> Option<bool> {
        let flag =
            unsafe { barvinok_sys::isl_map_is_equal(self.handle.as_ptr(), other.handle.as_ptr()) };
        isl_bool_to_optional_bool(flag)
    }
    pub fn disjoint(&self, other: &Self) -> Option<bool> {
        let flag = unsafe {
            barvinok_sys::isl_map_is_disjoint(self.handle.as_ptr(), other.handle.as_ptr())
        };
        isl_bool_to_optional_bool(flag)
    }
    map_transform!(lexmin, isl_map_lexmin);
    map_transform!(lexmax, isl_map_lexmax);
    map_transform!(range_reverse, isl_map_range_reverse);
    map_transform!(union, isl_map_union, [managed] map: Map<'a>);
    map_transform!(disjoint_union, isl_map_union_disjoint, [managed] map: Map<'a>);
    map_transform!(intersect_domain, isl_map_intersect_domain, [managed] set: Set<'a>);
    map_transform!(intersect_range, isl_map_intersect_range, [managed] set: Set<'a>);
    map_transform!(intersect_domain_factor_domain, isl_map_intersect_domain_factor_domain, [managed] map: Map<'a>);
    map_transform!(intersect_range_factor_range, isl_map_intersect_range_factor_range, [managed] map: Map<'a>);
    map_transform!(intersect_range_factor_domain, isl_map_intersect_range_factor_domain, [managed] map: Map<'a>);
    map_transform!(intersect_domain_factor_range, isl_map_intersect_domain_factor_range, [managed] map: Map<'a>);
    map_transform!(intersect_domain_wrapped_domain, isl_map_intersect_domain_wrapped_domain, [managed] set: Set<'a>);
    map_transform!(intersect_range_wrapped_range, isl_map_intersect_range_wrapped_domain, [managed] set: Set<'a>);
    map_transform!(apply_domain, isl_map_apply_domain, [managed] map: Map<'a>);
    map_transform!(apply_range, isl_map_apply_range, [managed] map: Map<'a>);
    map_transform!(product, isl_map_product, [managed] map: Map<'a>);
    map_transform!(domain_product, isl_map_domain_product, [managed] map: Map<'a>);
    map_transform!(range_product, isl_map_range_product, [managed] map: Map<'a>);
    map_transform!(flat_product, isl_map_flat_product, [managed] map: Map<'a>);
    map_transform!(flat_domain_product, isl_map_flat_domain_product, [managed] map: Map<'a>);
    map_transform!(flat_range_product, isl_map_flat_range_product, [managed] map: Map<'a>);
    map_flag!(map_domain_is_wrapping => domain_is_wrapping);
    map_flag!(map_range_is_wrapping => range_is_wrapping);
    map_flag!(map_is_product => is_product);
    map_transform!(factor_domain, isl_map_factor_domain);
    map_transform!(factor_range, isl_map_factor_range);
    map_transform!(domain_factor_domain, isl_map_domain_factor_domain);
    map_transform!(range_factor_range, isl_map_range_factor_range);
    map_transform!(domain_factor_range, isl_map_domain_factor_range);
    map_transform!(range_factor_domain, isl_map_range_factor_domain);
    map_transform!(intersect, isl_map_intersect, [managed] map: Map<'a>);
    map_transform!(intersect_params, isl_map_intersect_params, [managed] set: Set<'a>);
    map_transform!(subtract, isl_map_subtract, [managed] map: Map<'a>);
    map_transform!(subtract_domain, isl_map_subtract_domain, [managed] set: Set<'a>);
    map_transform!(subtract_range, isl_map_subtract_range, [managed] set: Set<'a>);
    map_transform!(complement, isl_map_complement);
    map_transform!(fix_input_si, isl_map_fix_input_si, [trivial] input: u32, [trivial] value: i32);
    map_transform!(fix_si, isl_map_fix_si, [trivial] dim_type: DimType, [trivial] pos: u32, [trivial] value: i32);
    map_transform!(fix_val, isl_map_fix_val, [trivial] dim_type: DimType, [trivial] pos: u32, [managed] value: Value<'a>);
    map_transform!(lower_bound_si, isl_map_lower_bound_si, [trivial] dim_type: DimType, [trivial] pos: u32, [trivial] value: i32);
    map_transform!(lower_bound_val, isl_map_lower_bound_val, [trivial] dim_type: DimType, [trivial] pos: u32, [managed] value: Value<'a>);
    map_transform!(upper_bound_si, isl_map_upper_bound_si, [trivial] dim_type: DimType, [trivial] pos: u32, [trivial] value: i32);
    map_transform!(upper_bound_val, isl_map_upper_bound_val, [trivial] dim_type: DimType, [trivial] pos: u32, [managed] value: Value<'a>);
    pub fn deltas(self) -> Result<Set<'a>, crate::Error> {
        let ctx = self.context_ref();
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_map_deltas(this.handle.as_ptr()) };
        NonNull::new(handle)
            .ok_or_else(|| ctx.last_error_or_unknown().into())
            .map(|handle| Set {
                handle,
                marker: std::marker::PhantomData,
            })
    }
    map_transform!(deltas_map, isl_map_deltas_map);
    map_transform!(detect_equalities, isl_map_detect_equalities);
    map_transform!(add_dims, isl_map_add_dims, [trivial] dim_type: DimType, [trivial] num: u32);
    map_transform!(insert_dims, isl_map_insert_dims, [trivial] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    map_transform!(move_dims, isl_map_move_dims, [trivial] dim_type: DimType, [trivial] dst_type: DimType, [trivial] dst_pos: u32, [trivial] src_pos: u32, [trivial] num: u32);
    map_transform!(project_out_param_id, isl_map_project_out_param_id, [managed] ident: Ident<'a>);
    map_transform!(project_out, isl_map_project_out, [trivial] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    map_transform!(project_out_all_params, isl_map_project_out_all_params);
    map_transform!(remove_unknown_divs, isl_map_remove_unknown_divs);
    map_transform!(remove_divs, isl_map_remove_divs);
    map_transform!(eliminate, isl_map_eliminate, [trivial] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    map_transform!(remove_dims, isl_map_remove_dims, [trivial] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    map_transform!(remove_divs_involving_dims, isl_map_remove_divs_involving_dims, [trivial] dim_type: DimType, [trivial] pos: u32, [trivial] num: u32);
    map_transform!(remove_inputs, isl_map_remove_inputs, [trivial] pos: u32, [trivial] num: u32);
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
