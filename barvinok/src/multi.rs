//! High-level abstractions for isl's "multi" object families.
//!
//! The concrete object wrappers remain generated, but the common behavior is
//! surfaced through traits so code can treat shared `isl_multi_*` families the
//! same way it treats `List<T>`.

use crate::{DimType, Result, space::Space};

pub trait Multi<'a>: Sized {
    type Element;

    fn space(&self) -> Result<Space<'a>>;
    fn component_count(&self) -> Result<u32>;
    fn component(&self, pos: usize) -> Result<Self::Element>;

    fn len(&self) -> Result<usize> {
        self.component_count().map(|count| count as usize)
    }

    fn is_empty(&self) -> Result<bool> {
        self.component_count().map(|count| count == 0)
    }

    fn first_component(&self) -> Result<Option<Self::Element>> {
        if self.is_empty()? {
            Ok(None)
        } else {
            self.component(0).map(Some)
        }
    }

    fn last_component(&self) -> Result<Option<Self::Element>> {
        let len = self.len()?;
        if len == 0 {
            Ok(None)
        } else {
            self.component(len - 1).map(Some)
        }
    }

    fn components(&self) -> Result<Vec<Self::Element>> {
        (0..self.len()?).map(|pos| self.component(pos)).collect()
    }
}

pub trait DomainMulti<'a>: Multi<'a> {
    fn domain_space(&self) -> Result<Space<'a>>;
}

pub trait MutableMulti<'a>: Multi<'a> {
    fn set_component(self, pos: usize, element: Self::Element) -> Result<Self>;
}

pub trait MultiList<'a>: Multi<'a> {
    type ElementList;

    fn list(&self) -> Result<Self::ElementList>;
}

macro_rules! impl_generated_multi_traits {
    ($ty:ty, element = $element:ty, list = $list:ty) => {
        impl<'a> Multi<'a> for $ty {
            type Element = $element;

            fn space(&self) -> Result<Space<'a>> {
                self.get_space()
            }

            fn component_count(&self) -> Result<u32> {
                self.size()
            }

            fn component(&self, pos: usize) -> Result<Self::Element> {
                self.get_at(pos as i32)
            }
        }

        impl<'a> DomainMulti<'a> for $ty {
            fn domain_space(&self) -> Result<Space<'a>> {
                self.get_domain_space()
            }
        }

        impl<'a> MutableMulti<'a> for $ty {
            fn set_component(self, pos: usize, element: Self::Element) -> Result<Self> {
                self.set_at(pos as i32, element)
            }
        }

        impl<'a> MultiList<'a> for $ty {
            type ElementList = $list;

            fn list(&self) -> Result<Self::ElementList> {
                self.get_list()
            }
        }
    };
}

impl_generated_multi_traits!(
    crate::multi_aff::MultiAffine<'a>,
    element = crate::aff::Affine<'a>,
    list = crate::list::AffineList<'a>
);
impl_generated_multi_traits!(
    crate::multi_id::MultiId<'a>,
    element = crate::ident::Ident<'a>,
    list = crate::list::IdentList<'a>
);
impl_generated_multi_traits!(
    crate::multi_pw_aff::MultiPiecewiseAffine<'a>,
    element = crate::pw_aff::PiecewiseAffine<'a>,
    list = crate::list::PiecewiseAffineList<'a>
);
impl_generated_multi_traits!(
    crate::multi_val::MultiValue<'a>,
    element = crate::value::Value<'a>,
    list = crate::list::ValueList<'a>
);

impl<'a> Multi<'a> for crate::pw_multi_aff::PiecewiseMultiAffine<'a> {
    type Element = crate::pw_aff::PiecewiseAffine<'a>;

    fn space(&self) -> Result<Space<'a>> {
        self.get_space()
    }

    fn component_count(&self) -> Result<u32> {
        self.dim(DimType::Out)
    }

    fn component(&self, pos: usize) -> Result<Self::Element> {
        self.get_at(pos as i32)
    }
}

impl<'a> DomainMulti<'a> for crate::pw_multi_aff::PiecewiseMultiAffine<'a> {
    fn domain_space(&self) -> Result<Space<'a>> {
        self.get_domain_space()
    }
}
