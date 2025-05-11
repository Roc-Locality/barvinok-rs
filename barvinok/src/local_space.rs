use std::{ffi::CStr, mem::ManuallyDrop, ptr::NonNull};

use barvinok_sys::isl_dim_type;

use crate::{
    DimType,
    aff::Affine,
    ident::Ident,
    impl_isl_handle,
    map::BasicMap,
    nonnull_or_alloc_error,
    space::Space,
    stat::{isl_bool_to_optional_bool, isl_size_to_optional_u32},
};

impl_isl_handle!([printer] LocalSpace, local_space);

impl<'a> TryFrom<Space<'a>> for LocalSpace<'a> {
    fn try_from(space: Space<'a>) -> Result<Self, crate::Error> {
        let ctx = space.context_ref();
        let space = ManuallyDrop::new(space);
        let handle = unsafe { barvinok_sys::isl_local_space_from_space(space.handle.as_ptr()) };
        let handle = NonNull::new(handle).ok_or_else(|| ctx.last_error_or_unknown())?;
        Ok(Self {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    type Error = crate::Error;
}

impl<'a> LocalSpace<'a> {
    pub fn is_params(&self) -> Option<bool> {
        let flag = unsafe { barvinok_sys::isl_local_space_is_params(self.handle.as_ptr()) };
        isl_bool_to_optional_bool(flag)
    }
    pub fn is_set(&self) -> Option<bool> {
        let flag = unsafe { barvinok_sys::isl_local_space_is_set(self.handle.as_ptr()) };
        isl_bool_to_optional_bool(flag)
    }
    pub fn set_tuple_id(&mut self, dim: DimType, id: Ident<'a>) {
        let id = ManuallyDrop::new(id);
        let handle = unsafe {
            barvinok_sys::isl_local_space_set_tuple_id(
                self.handle.as_ptr(),
                dim as isl_dim_type,
                id.handle.as_ptr(),
            )
        };
        let handle = nonnull_or_alloc_error(handle);
        self.handle = handle;
    }
    pub fn dim(&self, dim: DimType) -> Option<u32> {
        let dim =
            unsafe { barvinok_sys::isl_local_space_dim(self.handle.as_ptr(), dim as isl_dim_type) };
        isl_size_to_optional_u32(dim)
    }
    pub fn has_dim_name(&self, dim: DimType, pos: u32) -> Option<bool> {
        let flag = unsafe {
            barvinok_sys::isl_local_space_has_dim_name(
                self.handle.as_ptr(),
                dim as isl_dim_type,
                pos,
            )
        };
        isl_bool_to_optional_bool(flag)
    }
    pub fn get_dim_name(&self, dim: DimType, pos: u32) -> Result<Option<&str>, crate::Error> {
        let name = unsafe {
            barvinok_sys::isl_local_space_get_dim_name(
                self.handle.as_ptr(),
                dim as isl_dim_type,
                pos,
            )
        };
        if name.is_null() {
            Ok(None)
        } else {
            let cstr = unsafe { CStr::from_ptr(name) };
            let name = cstr.to_str()?;
            Ok(Some(name))
        }
    }
    pub fn set_dim_name(&mut self, dim: DimType, pos: u32, name: &str) -> Result<(), crate::Error> {
        let name = std::ffi::CString::new(name)?;
        let name = name.as_ptr();
        let handle = unsafe {
            barvinok_sys::isl_local_space_set_dim_name(
                self.handle.as_ptr(),
                dim as isl_dim_type,
                pos,
                name,
            )
        };
        let handle = nonnull_or_alloc_error(handle);
        self.handle = handle;
        Ok(())
    }
    pub fn has_dim_id(&self, dim: DimType, pos: u32) -> Option<bool> {
        let flag = unsafe {
            barvinok_sys::isl_local_space_has_dim_id(self.handle.as_ptr(), dim as isl_dim_type, pos)
        };
        isl_bool_to_optional_bool(flag)
    }
    pub fn get_dim_id(&self, dim: DimType, pos: u32) -> Option<Ident<'a>> {
        let id = unsafe {
            barvinok_sys::isl_local_space_get_dim_id(self.handle.as_ptr(), dim as isl_dim_type, pos)
        };
        NonNull::new(id).map(|id| Ident {
            handle: id,
            marker: std::marker::PhantomData,
        })
    }
    pub fn set_dim_id(&mut self, dim: DimType, pos: u32, id: Ident<'a>) {
        let id = ManuallyDrop::new(id);
        let handle = unsafe {
            barvinok_sys::isl_local_space_set_dim_id(
                self.handle.as_ptr(),
                dim as isl_dim_type,
                pos,
                id.handle.as_ptr(),
            )
        };
        let handle = nonnull_or_alloc_error(handle);
        self.handle = handle;
    }
    pub fn get_space(&self) -> Space<'a> {
        let handle = unsafe { barvinok_sys::isl_local_space_get_space(self.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Space {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn get_div(&self, pos: u32) -> Option<Affine<'a>> {
        let id = unsafe { barvinok_sys::isl_local_space_get_div(self.handle.as_ptr(), pos as i32) };
        NonNull::new(id).map(|id| Affine {
            handle: id,
            marker: std::marker::PhantomData,
        })
    }
    pub fn find_dim_by_name(&self, dim: DimType, name: &str) -> Option<u32> {
        let name = std::ffi::CString::new(name).ok()?;
        let pos = unsafe {
            barvinok_sys::isl_local_space_find_dim_by_name(
                self.handle.as_ptr(),
                dim as isl_dim_type,
                name.as_ptr(),
            )
        };
        isl_size_to_optional_u32(pos)
    }
    pub fn domain(self) -> Option<LocalSpace<'a>> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_local_space_domain(this.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| LocalSpace {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn range(self) -> Option<LocalSpace<'a>> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_local_space_range(this.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| LocalSpace {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn new_from_domain(dom: Self) -> Self {
        let dom = ManuallyDrop::new(dom);
        let handle = unsafe { barvinok_sys::isl_local_space_from_domain(dom.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn add_dims(&mut self, dim: DimType, num: u32) {
        let handle = unsafe {
            barvinok_sys::isl_local_space_add_dims(self.handle.as_ptr(), dim as isl_dim_type, num)
        };
        let handle = nonnull_or_alloc_error(handle);
        self.handle = handle;
    }
    pub fn drop_dims(&mut self, dim: DimType, from: u32, num: u32) {
        let handle = unsafe {
            barvinok_sys::isl_local_space_drop_dims(
                self.handle.as_ptr(),
                dim as isl_dim_type,
                from,
                num,
            )
        };
        let handle = nonnull_or_alloc_error(handle);
        self.handle = handle;
    }
    pub fn insert_dims(&mut self, dim: DimType, pos: u32, num: u32) {
        let handle = unsafe {
            barvinok_sys::isl_local_space_insert_dims(
                self.handle.as_ptr(),
                dim as isl_dim_type,
                pos,
                num,
            )
        };
        let handle = nonnull_or_alloc_error(handle);
        self.handle = handle;
    }
    pub fn new_set_from_params(params: Self) -> Self {
        let params = ManuallyDrop::new(params);
        let handle =
            unsafe { barvinok_sys::isl_local_space_set_from_params(params.handle.as_ptr()) };
        let handle = nonnull_or_alloc_error(handle);
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
    pub fn insersect(self, other: Self) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let other = ManuallyDrop::new(other);
        let handle = unsafe {
            barvinok_sys::isl_local_space_intersect(this.handle.as_ptr(), other.handle.as_ptr())
        };
        NonNull::new(handle).map(|handle| Self {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn wrap(self) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_local_space_wrap(this.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| Self {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn equal(&self, other: &Self) -> Option<bool> {
        let flag = unsafe {
            barvinok_sys::isl_local_space_is_equal(self.handle.as_ptr(), other.handle.as_ptr())
        };
        isl_bool_to_optional_bool(flag)
    }
    pub fn lifting(self) -> Option<BasicMap<'a>> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_local_space_lifting(this.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| BasicMap {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn flatten_domain(self) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_local_space_flatten_domain(this.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| Self {
            handle,
            marker: std::marker::PhantomData,
        })
    }
    pub fn flatten_range(self) -> Option<Self> {
        let this = ManuallyDrop::new(self);
        let handle = unsafe { barvinok_sys::isl_local_space_flatten_range(this.handle.as_ptr()) };
        NonNull::new(handle).map(|handle| Self {
            handle,
            marker: std::marker::PhantomData,
        })
    }
}

impl PartialEq for LocalSpace<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.equal(other).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_local_space() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 2, 3, 4).unwrap();
            let local_space = LocalSpace::try_from(space.clone()).unwrap();
            assert_eq!(
                local_space.get_space().handle.as_ptr(),
                space.handle.as_ptr()
            );
            assert_eq!(local_space.is_params(), Some(false));
            assert_eq!(local_space.is_set(), Some(false));
            assert_eq!(local_space.dim(DimType::Param), Some(2));
            assert_eq!(local_space.dim(DimType::In), Some(3));
            assert_eq!(local_space.dim(DimType::Out), Some(4));
            println!("{:?}", local_space);
        });
    }

    #[test]
    fn test_lifting_set_space() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 3).unwrap();
            let local_space = LocalSpace::try_from(space.clone()).unwrap();
            let lifted_space = local_space.lifting().unwrap();
            println!("{:?}", lifted_space);
        });
    }

    #[test]
    fn test_mutate_shapes() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 2, 3, 4).unwrap();
            let mut local_space = LocalSpace::try_from(space.clone()).unwrap();
            local_space.add_dims(DimType::Param, 2);
            local_space.set_tuple_id(DimType::In, Ident::new(ctx, "input").unwrap());
            local_space.set_dim_name(DimType::Out, 0, "y").unwrap();
            println!("{:?}", local_space);
            assert!(local_space.get_dim_id(DimType::In, 0).is_none());
            assert!(local_space.get_dim_id(DimType::Out, 0).is_some());
            assert_eq!(
                local_space.get_dim_name(DimType::Out, 0).unwrap(),
                Some("y")
            );
        });
    }
}
