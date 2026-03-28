use crate::DimType;

include!(concat!(env!("OUT_DIR"), "/generated/space.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;
    use crate::ident::Ident;

    #[test]
    fn test_space_creation() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::new(ctx, 2, 4, 3);
            println!("{:?}", space);
        });
    }

    #[test]
    fn test_space_params() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::params(ctx, 2);
            println!("{:?}", space);
        });
    }

    #[test]
    fn test_space_unit() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::unit(ctx);
            println!("{:?}", space);
        });
    }

    #[test]
    fn test_space_set() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 4);
            println!("{:?}", space);
        });
    }

    #[test]
    fn test_space_add_param() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 4).unwrap();
            let id = Ident::new(ctx, "x").unwrap();
            let space = space.add_param(id).unwrap();
            println!("{:?}", space);
        });
    }

    #[test]
    fn test_space_set_tuple_name() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 4).unwrap();
            let space = space.set_tuple_name(DimType::In, "input").unwrap();
            println!("{:?}", space);
            assert!(space.has_tuple_name(DimType::In).unwrap());
            assert_eq!(space.get_tuple_name(DimType::In).unwrap(), Some("input"));
        });
    }

    #[test]
    fn test_space_add_dims() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 4).unwrap();
            assert!(space.is_set().unwrap());
            let space = space.add_dims(DimType::In, 2).unwrap();
            println!("{:?}", space);
            assert!(space.is_map().unwrap());
        });
    }
}
