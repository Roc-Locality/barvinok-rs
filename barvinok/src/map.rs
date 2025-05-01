use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{constraint::Constraint, impl_isl_handle};

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

#[cfg(test)]
mod tests {
    use crate::{
        Context, DimType, constraint::Constraint, local_space::LocalSpace, map::BasicMap,
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
}
