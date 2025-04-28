use std::mem::ManuallyDrop;

use crate::{constraint::Constraint, impl_isl_handle, nonnull_or_alloc_error};

impl_isl_handle!(Map, map);
impl_isl_handle!(BasicMap, basic_map);

impl<'a> From<Constraint<'a>> for BasicMap<'a> {
    fn from(constraint: Constraint<'a>) -> Self {
        let constraint = ManuallyDrop::new(constraint);
        let handle =
            unsafe { barvinok_sys::isl_basic_map_from_constraint(constraint.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        BasicMap {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}
