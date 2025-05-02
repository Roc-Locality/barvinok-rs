use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::aff::Affine;
use crate::local_space::LocalSpace;
use crate::space::Space;
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
