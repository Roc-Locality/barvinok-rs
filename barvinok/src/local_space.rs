use crate::{DimType, impl_isl_handle, space::Space};

impl_isl_handle!([printer] LocalSpace, local_space);

include!(concat!(env!("OUT_DIR"), "/generated/local_space.rs"));

impl<'a> TryFrom<Space<'a>> for LocalSpace<'a> {
    type Error = crate::Error;

    fn try_from(space: Space<'a>) -> Result<Self, Self::Error> {
        Self::from_space(space)
    }
}

impl PartialEq for LocalSpace<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, ident::Ident};

    #[test]
    fn test_local_space() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 2, 3, 4).unwrap();
            let local_space = LocalSpace::from_space(space.clone()).unwrap();
            assert_eq!(
                local_space.get_space().unwrap().handle.as_ptr(),
                space.handle.as_ptr()
            );
            assert!(!local_space.is_params().unwrap());
            assert!(!local_space.is_set().unwrap());
            assert_eq!(local_space.dim(DimType::Param).unwrap(), 2);
            assert_eq!(local_space.dim(DimType::In).unwrap(), 3);
            assert_eq!(local_space.dim(DimType::Out).unwrap(), 4);
            println!("{:?}", local_space);
        });
    }

    #[test]
    fn test_lifting_set_space() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 3).unwrap();
            let local_space = LocalSpace::from_space(space.clone()).unwrap();
            let lifted_space = local_space.lifting().unwrap();
            println!("{:?}", lifted_space);
        });
    }

    #[test]
    fn test_mutate_shapes() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 2, 3, 4).unwrap();
            let local_space = LocalSpace::from_space(space.clone())
                .unwrap()
                .add_dims(DimType::Param, 2)
                .unwrap()
                .set_tuple_id(DimType::In, Ident::new(ctx, "input").unwrap())
                .unwrap()
                .set_dim_name(DimType::Out, 0, "y")
                .unwrap();
            println!("{:?}", local_space);
            assert!(local_space.get_dim_id(DimType::In, 0).is_none());
            assert!(local_space.get_dim_id(DimType::Out, 0).is_some());
            assert_eq!(
                local_space.get_dim_name(DimType::Out, 0).unwrap(),
                Some("y")
            );
        });
    }

    #[test]
    fn test_generated_local_space_methods() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let params = LocalSpace::from_space(Space::params(ctx, 1).unwrap()).unwrap();
            let set_space = params.clone().set_from_params().unwrap();
            assert!(set_space.get_space().unwrap().is_set().unwrap());

            let intersected = set_space.clone().intersect(set_space).unwrap();
            assert!(
                intersected
                    .is_equal(&params.set_from_params().unwrap())
                    .unwrap()
            );
        });
    }
}
