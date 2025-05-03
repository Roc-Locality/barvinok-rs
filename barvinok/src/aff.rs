use crate::local_space::LocalSpace;
use crate::space::Space;
use crate::value::Value;
use crate::{impl_isl_handle, isl_ctor};
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

impl_isl_handle!(Affine, aff);

impl<'a> Affine<'a> {
    isl_ctor!(zero_on_domain_space, isl_aff_zero_on_domain_space, space: Space<'a>);
    isl_ctor!(zero_on_domain, isl_aff_zero_on_domain, space: LocalSpace<'a>);
    isl_ctor!(val_on_domain_space, isl_aff_val_on_domain_space, space: Space<'a>, [managed] val: Value<'a>);
}
