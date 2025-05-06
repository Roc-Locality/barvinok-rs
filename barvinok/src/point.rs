use crate::{
    DimType, impl_isl_handle, isl_ctor, isl_project, isl_transform, space::Space, value::Value,
};
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

impl_isl_handle!(Point, point);

impl<'a> Point<'a> {
    isl_project!([into(Space)] get_space, isl_point_get_space);
    isl_ctor!(zero, isl_point_zero, space : Space<'a>);
    isl_project!([into(Value)] get_coordinate_val, isl_point_get_coordinate_val, [cast(u32)] dim_type : DimType, [cast(i32)] pos : u32);
    isl_transform!(set_coordinate_val, isl_point_set_coordinate_val, [cast(u32)] dim_type : DimType, [cast(i32)] pos : u32, [managed] value : Value<'a>);
    isl_transform!(add_ui, isl_point_add_ui, [cast(u32)] dim_type : DimType, [cast(i32)] pos : u32, [trivial] value : u32);
    isl_transform!(sub_ui, isl_point_sub_ui, [cast(u32)] dim_type : DimType, [cast(i32)] pos : u32, [trivial] value : u32);
    isl_ctor!(void, isl_point_void, space : Space<'a>);
}
