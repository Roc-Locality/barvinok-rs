use crate::ident::Ident;
use crate::local_space::LocalSpace;
use crate::space::Space;
use crate::stat::{isl_bool_to_optional_bool, isl_size_to_optional_u32};
use crate::value::Value;
use crate::{
    DimType, impl_isl_handle, isl_ctor, isl_flag, isl_project, isl_size, isl_str, isl_transform,
};
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

impl_isl_handle!(Affine, aff);

impl<'a> Affine<'a> {
    isl_ctor!(zero_on_domain_space, isl_aff_zero_on_domain_space, space: Space<'a>);
    isl_ctor!(zero_on_domain, isl_aff_zero_on_domain, space: LocalSpace<'a>);
    isl_ctor!(val_on_domain_space, isl_aff_val_on_domain_space, space: Space<'a>, [managed] val: Value<'a>);
    isl_ctor!(val_on_domain, isl_aff_val_on_domain, space: LocalSpace<'a>, [managed] val: Value<'a>);
    isl_ctor!(var_on_domain, isl_aff_var_on_domain, space: LocalSpace<'a>, [cast(u32)] dim_type: DimType, [trivial] pos: u32);
    isl_ctor!(nan_on_domain_space, isl_aff_nan_on_domain_space, space: Space<'a>);
    isl_ctor!(nan_on_domain, isl_aff_nan_on_domain, space: LocalSpace<'a>);
    isl_ctor!(param_on_domain_space_id, isl_aff_param_on_domain_space_id, space: Space<'a>, [managed] id: Ident<'a>);
    isl_flag!(aff_involves_locals => involves_locals);
    isl_size!(aff_dim => dim, [cast(u32)] dim_type: DimType);
    isl_project!([into(Space)] get_domain_space, isl_aff_get_domain_space);
    isl_project!([into(Space)] get_space, isl_aff_get_space);
    isl_project!([into(LocalSpace)] get_domain_local_space, isl_aff_get_domain_local_space);
    isl_project!([into(LocalSpace)] get_local_space, isl_aff_get_local_space);
    isl_str!(aff_get_dim_name => get_dim_name, [cast(u32)] dim_type: DimType, [trivial] pos: u32);
    isl_project!([into(Value)] get_constant_val, isl_aff_get_constant_val);
    isl_project!([into(Value)] get_coefficient_val, isl_aff_get_coefficient_val, [cast(u32)] dim_type: DimType, [cast(i32)] pos: u32);
    isl_project!([into(Value)] get_denominator_val, isl_aff_get_denominator_val);
    isl_transform!(set_constant_si, isl_aff_set_constant_si, [trivial] constant: i32);
    isl_transform!(set_constant_val, isl_aff_set_constant_val, [managed] constant: Value<'a>);
    isl_transform!(set_coefficient_si, isl_aff_set_coefficient_si, [cast(u32)] dim_type: DimType, [cast(i32)] pos: u32, [trivial] coefficient: i32);
    isl_transform!(set_coefficient_val, isl_aff_set_coefficient_val, [cast(u32)] dim_type: DimType, [cast(i32)] pos: u32, [managed] coefficient: Value<'a>);
    isl_transform!(add_constant_si, isl_aff_add_constant_si, [trivial] constant: i32);
    isl_transform!(add_constant_val, isl_aff_add_constant_val, [managed] constant: Value<'a>);
    isl_transform!(add_constant_num_si, isl_aff_add_constant_num_si, [trivial] constant: i32);
    isl_transform!(add_coefficient_si, isl_aff_add_coefficient_si, [cast(u32)] dim_type: DimType, [cast(i32)] pos: u32, [trivial] coefficient: i32);
    isl_transform!(add_coefficient_val, isl_aff_add_coefficient_val, [cast(u32)] dim_type: DimType, [cast(i32)] pos: u32, [managed] coefficient: Value<'a>);
    isl_flag!(aff_is_cst => is_cst);
    isl_transform!(set_tuple_id, isl_aff_set_tuple_id, [cast(u32)] dim_type: DimType, [managed] id: Ident<'a>);
    isl_transform!(set_dim_name, isl_aff_set_dim_name, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [str] name: &str);
    isl_transform!(set_dim_id, isl_aff_set_dim_id, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [managed] id: Ident<'a>);
    isl_flag!(aff_plain_is_equal => plain_is_equal, [ref] other: &Affine<'a>);
    isl_flag!(aff_plain_is_zero => plain_is_zero);
    isl_flag!(aff_is_nan => plain_is_nan);
    isl_project!([into(Affine)] get_div, isl_aff_get_div, [cast(i32)] pos: u32);
    isl_ctor!(from_range, isl_aff_from_range, range: Affine<'a>);
    isl_transform!(checked_neg, isl_aff_neg);
    isl_transform!(checked_ceil, isl_aff_ceil);
    isl_transform!(floor, isl_aff_floor);
    isl_transform!(mod_val, isl_aff_mod_val, [managed] val: Value<'a>);
    isl_transform!(checked_mul, isl_aff_mul, [managed] aff: Affine<'a>);
    isl_transform!(checked_add, isl_aff_add, [managed] aff: Affine<'a>);
    isl_transform!(checked_sub, isl_aff_sub, [managed] aff: Affine<'a>);
    isl_transform!(checked_div, isl_aff_div, [managed] aff: Affine<'a>);
    isl_transform!(scale_val, isl_aff_scale_val, [managed] val: Value<'a>);
    isl_transform!(scale_down_ui, isl_aff_scale_down_ui, [trivial] scale: u32);
    isl_transform!(scale_down_val, isl_aff_scale_down_val, [managed] scale: Value<'a>);
    isl_transform!(domain_reverse, isl_aff_domain_reverse);
}

impl<'a> std::ops::Neg for Affine<'a> {
    type Output = Affine<'a>;
    fn neg(self) -> Self::Output {
        self.checked_neg().unwrap()
    }
}

impl<'a> std::ops::Add for Affine<'a> {
    type Output = Affine<'a>;
    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs).unwrap()
    }
}

impl<'a> std::ops::Sub for Affine<'a> {
    type Output = Affine<'a>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs).unwrap()
    }
}

impl<'a> std::ops::Mul for Affine<'a> {
    type Output = Affine<'a>;
    fn mul(self, rhs: Self) -> Self::Output {
        self.checked_mul(rhs).unwrap()
    }
}
impl<'a> std::ops::Div for Affine<'a> {
    type Output = Affine<'a>;
    fn div(self, rhs: Self) -> Self::Output {
        self.checked_mul(rhs).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;
    use crate::space::Space;

    #[test]
    fn test_aff() -> anyhow::Result<()> {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 2)?;
            let aff = Affine::zero_on_domain_space(space)?;
            println!("Affine: {:?}", aff);
            Ok(())
        })
    }

    #[test]
    fn test_aff_with_name() -> anyhow::Result<()> {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 2)?;
            let aff = Affine::zero_on_domain_space(space)?
                .set_dim_name(DimType::Param, 0, "N")?
                .set_dim_name(DimType::Param, 1, "M")?
                .set_dim_name(DimType::In, 0, "i")?
                .set_dim_name(DimType::In, 1, "j")?;
            println!("Affine: {:?}", aff);
            Ok(())
        })
    }

    #[test]
    fn test_aff_binary() -> anyhow::Result<()> {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 2)?;
            let local_space = LocalSpace::try_from(space).unwrap();
            let aff1 = Affine::var_on_domain(local_space.clone(), DimType::Param, 0)?;
            let aff2 = Affine::var_on_domain(local_space.clone(), DimType::Out, 1)?;
            let aff3 = Affine::zero_on_domain(local_space)?;
            let aff4 = aff1 + aff2;
            let aff5 = aff4 + aff3;
            println!("Affine: {:?}", aff5);
            Ok(())
        })
    }
}
