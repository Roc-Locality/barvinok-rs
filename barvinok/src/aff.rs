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
    isl_transform!(set_dim_name, isl_aff_set_dim_name, [cast(u32)] dim_type: DimType, [trivial] pos: u32, [str] name: &str);
}
