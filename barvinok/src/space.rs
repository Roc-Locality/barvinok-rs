use crate::stat::isl_size_to_optional_u32;
use crate::{DimType, impl_isl_handle, isl_ctor, isl_flag, isl_size, isl_str, isl_transform};
use crate::{ident::Ident, stat::isl_bool_to_optional_bool};
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

impl_isl_handle!(Space, space);

impl<'a> Space<'a> {
    isl_ctor!([ctx] new, isl_space_alloc, [trivial] num_params: u32, [trivial] num_inputs: u32, [trivial] num_outputs: u32);
    isl_ctor!([ctx] set, isl_space_set_alloc, [trivial] num_params: u32, [trivial] num_dims: u32);
    isl_ctor!([ctx] params, isl_space_params_alloc, [trivial] num_params: u32);
    isl_ctor!([ctx] unit, isl_space_unit);
    isl_flag!(space_is_params => is_params);
    isl_flag!(space_is_set => is_set);
    isl_flag!(space_is_map => is_map);
    isl_transform!(add_param_id, isl_space_add_param_id, [managed] id: Ident<'a>);
    isl_transform!(set_tuple_name, isl_space_set_tuple_name, [cast(u32)] dim_type: DimType, [str] name: &str);
    isl_flag!(space_has_tuple_name => has_tuple_name, [cast(u32)] dim_type: DimType);
    isl_str!(space_get_tuple_name => get_tuple_name, [cast(u32)] dim_type: DimType);
    isl_transform!(add_dims, isl_space_add_dims, [cast(u32)] dim_type: DimType, [trivial] num: u32);
    isl_size!(space_dim => get_dim, [cast(u32)] dim_type: DimType);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_space_creation() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 2, 4, 3);
            println!("{:?}", space);
        });
    }

    #[test]
    fn test_space_params() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::params(ctx, 2);
            println!("{:?}", space);
        });
    }

    #[test]
    fn test_space_unit() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::unit(ctx);
            println!("{:?}", space);
        });
    }

    #[test]
    fn test_space_set() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 4);
            println!("{:?}", space);
        });
    }

    #[test]
    fn test_space_add_param_id() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 4).unwrap();
            let id = Ident::new(ctx, "x").unwrap();
            let space = space.add_param_id(id).unwrap();
            println!("{:?}", space);
        });
    }

    #[test]
    fn test_space_set_tuple_name() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 4).unwrap();
            let space = space.set_tuple_name(DimType::In, "input").unwrap();
            println!("{:?}", space);
            assert!(space.has_tuple_name(DimType::In).unwrap());
            assert_eq!(space.get_tuple_name(DimType::In).unwrap(), "input");
        });
    }

    #[test]
    fn test_space_add_dims() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 4).unwrap();
            assert!(space.is_set().unwrap());
            let space = space.add_dims(DimType::In, 2).unwrap();
            println!("{:?}", space);
            assert!(space.is_map().unwrap());
        });
    }
}
