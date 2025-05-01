use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{
    ContextRef, DimType,
    constraint::Constraint,
    ident::Ident,
    impl_isl_handle,
    list::List,
    map::BasicMap,
    nonnull_or_alloc_error,
    polynomial::PiecewiseQuasiPolynomial,
    space::Space,
    stat::{isl_bool_to_optional_bool, isl_size_to_optional_u32},
};

impl_isl_handle!(Set, set);
impl_isl_handle!(BasicSet, basic_set);

macro_rules! basic_set_constructor {
    ($fn_name:ident, $isl_fn:ident) => {
        paste::paste! {
            pub fn [<new_ $fn_name>](space: Space<'a>) -> Result<Self, crate::Error> {
                let ctx = space.context_ref();
                let space = ManuallyDrop::new(space);
                let handle = unsafe { barvinok_sys::$isl_fn(space.handle.as_ptr()) };
                let handle = NonNull::new(handle).ok_or_else(|| ctx.last_error_or_unknown())?;
                Ok(Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
            }
        }
    };
}

macro_rules! basic_set_unary {
    ($fn_name:ident) => {
        paste::paste! {
            pub fn $fn_name(self) -> Result<Self, crate::Error> {
                let this = ManuallyDrop::new(self);
                let handle = unsafe { barvinok_sys::[<isl_basic_set_ $fn_name>](this.handle.as_ptr()) };
                let handle = nonnull_or_alloc_error(handle);
                Ok(Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
            }
        }
    };
}

macro_rules! basic_set_binary {
    ($fn_name:ident) => {
        paste::paste! {
            pub fn $fn_name(self, other: BasicSet<'a>) -> Result<Self, crate::Error> {
                let this = ManuallyDrop::new(self);
                let other = ManuallyDrop::new(other);
                let handle = unsafe {
                    barvinok_sys::[<isl_basic_set_ $fn_name>](this.handle.as_ptr(), other.handle.as_ptr())
                };
                let handle = nonnull_or_alloc_error(handle);
                Ok(Self {
                    handle,
                    marker: std::marker::PhantomData,
                })
            }
        }
    };
}

impl<'a> BasicSet<'a> {
    basic_set_constructor!(universe, isl_basic_set_universe);
    basic_set_constructor!(empty, isl_basic_set_empty);
    basic_set_constructor!(nat_universe, isl_basic_set_nat_universe);
    basic_set_constructor!(positive_orthant, isl_basic_set_positive_orthant);
    pub fn num_dims(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_n_dim(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn num_params(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_n_param(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn total_dims(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_total_dim(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn get_dims(&self, ty: DimType) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_basic_set_dim(self.handle.as_ptr(), ty as u32) };
        isl_size_to_optional_u32(num)
    }
    basic_set_binary!(intersect);
    basic_set_binary!(intersect_params);
    basic_set_unary!(affine_hull);
    basic_set_unary!(sample);
    basic_set_unary!(remove_redundancies);
    basic_set_unary!(detect_equalities);
    pub fn cardinality(self) -> Option<PiecewiseQuasiPolynomial<'a>> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_basic_set_card(this.handle.as_ptr()) };
        let handle = NonNull::new(handle)?;
        Some(PiecewiseQuasiPolynomial {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn add_constraint(self, constraint: Constraint<'a>) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let constraint = ManuallyDrop::new(constraint);
        let handle = unsafe {
            barvinok_sys::isl_basic_set_add_constraint(
                this.handle.as_ptr(),
                constraint.handle.as_ptr(),
            )
        };
        NonNull::new(handle).map(|handle| BasicSet {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn get_constraints(&self) -> Option<List<Constraint<'a>>> {
        let handle =
            unsafe { barvinok_sys::isl_basic_set_get_constraint_list(self.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| List {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn apply(self, map: BasicMap<'a>) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let map = ManuallyDrop::new(map);
        let handle =
            unsafe { barvinok_sys::isl_basic_set_apply(this.handle.as_ptr(), map.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| BasicSet {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn remove_dims(self, ty: DimType, first: u32, num: u32) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe {
            barvinok_sys::isl_basic_set_remove_dims(this.handle.as_ptr(), ty as u32, first, num)
        };
        NonNull::new(handle).map(|handle| BasicSet {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn from_str(ctx: ContextRef<'a>, str: &str) -> Result<Self, crate::Error> {
        let c_str = std::ffi::CString::new(str)?;
        let handle =
            unsafe { barvinok_sys::isl_basic_set_read_from_str(ctx.0.as_ptr(), c_str.as_ptr()) };
        let handle = NonNull::new(handle).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(BasicSet {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn is_rational(&self) -> Option<bool> {
        let is_rational = unsafe { barvinok_sys::isl_basic_set_is_rational(self.handle.as_ptr()) };
        isl_bool_to_optional_bool(is_rational)
    }
}

impl<'a> Set<'a> {
    pub fn num_dims(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_set_n_dim(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn num_params(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_set_n_param(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn tuple_dims(&self) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_set_tuple_dim(self.handle.as_ptr()) };
        isl_size_to_optional_u32(num)
    }
    pub fn get_dims(&self, ty: DimType) -> Option<u32> {
        let num = unsafe { barvinok_sys::isl_set_dim(self.handle.as_ptr(), ty as u32) };
        isl_size_to_optional_u32(num)
    }
    pub fn space(&self) -> Option<Space<'a>> {
        let handle = unsafe { barvinok_sys::isl_set_get_space(self.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| Space {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn reset_space(self, space: Space<'a>) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe {
            barvinok_sys::isl_set_reset_space(this.handle.as_ptr(), space.handle.as_ptr())
        };
        NonNull::new(handle).map(|handle| Set {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn has_tuple_name(&self) -> Option<bool> {
        let has = unsafe { barvinok_sys::isl_set_has_tuple_name(self.handle.as_ptr()) };
        isl_bool_to_optional_bool(has)
    }
    pub fn get_tuple_name(&self) -> Result<&str, crate::Error> {
        let ctx = self.context_ref();
        let name = unsafe { barvinok_sys::isl_set_get_tuple_name(self.handle.as_ptr()) };
        if name.is_null() {
            return Err(ctx.last_error_or_unknown().into());
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(name) };
        Ok(c_str.to_str()?)
    }
    pub fn set_tuple_name(self, name: &str) -> Result<Self, crate::Error> {
        let ctx = self.context_ref();
        let this = ManuallyDrop::new(self);
        let c_str = std::ffi::CString::new(name)?;
        let handle =
            unsafe { barvinok_sys::isl_set_set_tuple_name(this.handle.as_ptr(), c_str.as_ptr()) };
        let handle = NonNull::new(handle).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(Set {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn has_dim_name(&self, ty: DimType, pos: u32) -> Option<bool> {
        let has =
            unsafe { barvinok_sys::isl_set_has_dim_name(self.handle.as_ptr(), ty as u32, pos) };
        isl_bool_to_optional_bool(has)
    }
    pub fn get_dim_name(&self, ty: DimType, pos: u32) -> Result<&str, crate::Error> {
        let ctx = self.context_ref();
        let name =
            unsafe { barvinok_sys::isl_set_get_dim_name(self.handle.as_ptr(), ty as u32, pos) };
        if name.is_null() {
            return Err(ctx.last_error_or_unknown().into());
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(name) };
        Ok(c_str.to_str()?)
    }
    pub fn set_dim_name(self, ty: DimType, pos: u32, name: &str) -> Result<Self, crate::Error> {
        let ctx = self.context_ref();
        let this = ManuallyDrop::new(self);
        let c_str = std::ffi::CString::new(name)?;
        let handle = unsafe {
            barvinok_sys::isl_set_set_dim_name(this.handle.as_ptr(), ty as u32, pos, c_str.as_ptr())
        };
        let handle = NonNull::new(handle).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(Set {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn has_dim_id(&self, ty: DimType, pos: u32) -> Option<bool> {
        let has = unsafe { barvinok_sys::isl_set_has_dim_id(self.handle.as_ptr(), ty as u32, pos) };
        isl_bool_to_optional_bool(has)
    }
    pub fn get_dim_id(&self, ty: DimType, pos: u32) -> Result<Ident<'a>, crate::Error> {
        let ctx = self.context_ref();
        let id = unsafe { barvinok_sys::isl_set_get_dim_id(self.handle.as_ptr(), ty as u32, pos) };
        if id.is_null() {
            return Err(ctx.last_error_or_unknown().into());
        }
        let handle = NonNull::new(id).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(Ident {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn set_dim_id(self, ty: DimType, pos: u32, id: Ident<'a>) -> Result<Self, crate::Error> {
        let ctx = self.context_ref();
        let this = ManuallyDrop::new(self);
        let id = ManuallyDrop::new(id);
        let handle = unsafe {
            barvinok_sys::isl_set_set_dim_id(
                this.handle.as_ptr(),
                ty as u32,
                pos,
                id.handle.as_ptr(),
            )
        };
        let handle = NonNull::new(handle).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(Set {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn has_tuple_id(&self) -> Option<bool> {
        let has = unsafe { barvinok_sys::isl_set_has_tuple_id(self.handle.as_ptr()) };
        isl_bool_to_optional_bool(has)
    }
    pub fn get_tuple_id(&self) -> Result<Ident<'a>, crate::Error> {
        let ctx = self.context_ref();
        let id = unsafe { barvinok_sys::isl_set_get_tuple_id(self.handle.as_ptr()) };
        if id.is_null() {
            return Err(ctx.last_error_or_unknown().into());
        }
        let handle = NonNull::new(id).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(Ident {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn set_tuple_id(self, id: Ident<'a>) -> Result<Self, crate::Error> {
        let ctx = self.context_ref();
        let this = ManuallyDrop::new(self);
        let id = ManuallyDrop::new(id);
        let handle =
            unsafe { barvinok_sys::isl_set_set_tuple_id(this.handle.as_ptr(), id.handle.as_ptr()) };
        let handle = NonNull::new(handle).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(Set {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn reset_tuple_id(self) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_set_reset_tuple_id(this.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| Set {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn reset_user(self) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_set_reset_user(this.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| Set {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn find_dim_by_id(&self, ty: DimType, id: &Ident<'a>) -> Option<u32> {
        let id = ManuallyDrop::new(id);
        let num = unsafe {
            barvinok_sys::isl_set_find_dim_by_id(
                self.handle.as_ptr(),
                ty as u32,
                id.handle.as_ptr(),
            )
        };
        isl_size_to_optional_u32(num)
    }
    pub fn find_dim_by_name(&self, ty: DimType, name: &str) -> Result<u32, crate::Error> {
        let c_str = std::ffi::CString::new(name)?;
        let num = unsafe {
            barvinok_sys::isl_set_find_dim_by_name(self.handle.as_ptr(), ty as u32, c_str.as_ptr())
        };
        isl_size_to_optional_u32(num)
            .ok_or_else(|| self.context_ref().last_error_or_unknown().into())
    }
}

impl<'a> List<'a, BasicSet<'a>> {
    pub fn intersect(self) -> BasicSet<'a> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_basic_set_list_intersect(this.handle.as_ptr()) };
        let handle = NonNull::new(handle).unwrap();
        BasicSet {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a> List<'a, Set<'a>> {
    pub fn union(self) -> Set<'a> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_set_list_union(this.handle.as_ptr()) };
        let handle = NonNull::new(handle).unwrap();
        Set {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a> TryFrom<Constraint<'a>> for BasicSet<'a> {
    fn try_from(constraint: Constraint<'a>) -> Result<Self, crate::Error> {
        let ctx = constraint.context_ref();
        let constraint = ManuallyDrop::new(constraint);
        let handle =
            unsafe { barvinok_sys::isl_basic_set_from_constraint(constraint.handle.as_ptr()) };
        let handle = NonNull::new(handle).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(BasicSet {
            handle,
            marker: std::marker::PhantomData,
        })
    }

    type Error = crate::Error;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Context, local_space::LocalSpace};

    #[test]
    fn test_basic_set_creation() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 0, 0, 3);
            let basic_set = BasicSet::new_universe(space.clone()).unwrap();
            println!("{:?}", basic_set);
            let basic_set = BasicSet::new_empty(space.clone()).unwrap();
            println!("{:?}", basic_set);
            let basic_set = BasicSet::new_nat_universe(space.clone()).unwrap();
            println!("{:?}", basic_set);
            let basic_set = BasicSet::new_positive_orthant(space.clone()).unwrap();
            println!("{:?}", basic_set);
        });
    }

    #[test]
    fn test_basic_set_bin_ops() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 0, 0, 3);
            let basic_set1 = BasicSet::new_universe(space.clone()).unwrap();
            let basic_set2 = BasicSet::new_empty(space.clone()).unwrap();
            let basic_set3 = basic_set1.intersect(basic_set2).unwrap();
            println!("{:?}", basic_set3);
        });
    }

    #[test]
    fn test_basic_set_unary_ops() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 0, 0, 3);
            let basic_set = BasicSet::new_positive_orthant(space.clone()).unwrap();
            let basic_set = basic_set.affine_hull().unwrap();
            println!("{:?}", basic_set);
        });
    }
    #[test]
    fn test_basic_set_cardinality() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new_set(ctx, 1, 4);
            let basic_set = BasicSet::new_universe(space.clone()).unwrap();
            let card = basic_set.cardinality().unwrap();
            println!("{:?}", card);
        });
    }
    #[test]
    fn test_interval_product_space() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new_set(ctx, 3, 3);
            let local_space = LocalSpace::from(space.clone());
            let mut set = BasicSet::new_universe(space.clone()).unwrap();
            for i in 0..3 {
                {
                    let mut i_ge_0 = Constraint::new_inequality(local_space.clone());
                    i_ge_0 = i_ge_0.set_coefficient_si(DimType::Out, i, 1).unwrap();
                    set = set.add_constraint(i_ge_0).unwrap();
                    println!("{:?}", set);
                }
                {
                    let mut i_lt_p = Constraint::new_inequality(local_space.clone());
                    i_lt_p = i_lt_p.set_coefficient_si(DimType::Param, i, 1).unwrap();
                    i_lt_p = i_lt_p.set_coefficient_si(DimType::Out, i, -1).unwrap();
                    i_lt_p = i_lt_p.set_constant_si(-1).unwrap();
                    set = set.add_constraint(i_lt_p).unwrap();
                    println!("{:?}", set);
                }
            }
            let card = set.clone().cardinality().unwrap();
            println!("{:?}", card);
            println!("constraints:");
            let list = set.get_constraints().unwrap();
            for i in list.iter() {
                println!("{:?}", i);
            }
        });
    }

    #[test]
    fn test_basic_set_from_str() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let basic_set =
                BasicSet::from_str(ctx, "[p0, p1] -> { [i0, i1] : 5i0 + 6i1 >= p1 - p0 }").unwrap();
            println!("{:?}", basic_set);
        });
    }

    #[test]
    fn test_basic_set_list_intersect() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new_set(ctx, 1, 5);
            let basic_set1 = BasicSet::new_universe(space.clone()).unwrap();
            let basic_set2 = BasicSet::new_empty(space.clone()).unwrap();
            let mut list = List::new(ctx, 2);
            list.push(basic_set1);
            list.push(basic_set2);
            let intersected_set = list.intersect();
            println!("{:?}", intersected_set);
        });
    }
}
