use crate::impl_isl_handle;

impl_isl_handle!([printer] Vector, vec);

include!(concat!(env!("OUT_DIR"), "/generated/vec.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_vector_creation() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let vector = Vector::new(ctx, 5).unwrap();
            println!("{:?}", vector);
            assert_eq!(vector.size().unwrap(), 5);
        });
    }

    #[test]
    fn test_get_as_val() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let vector = Vector::new(ctx, 5).unwrap();
            assert!(vector.get_element_val(10).is_err());
            let val = vector.get_element_val(0).unwrap();
            assert!(val.is_zero().unwrap());
        });
    }

    #[test]
    fn test_set_si() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let vector = Vector::new(ctx, 5).unwrap().set_element_si(0, 10).unwrap();
            let val = vector.get_element_val(0).unwrap();
            assert_eq!(val.to_f64(), 10.0);
            assert!(vector.clone().set_element_si(99, 20).is_err());
        });
    }

    #[test]
    fn test_generated_vector_methods() {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let vector = Vector::zero(ctx, 2).unwrap().set_element_si(1, 3).unwrap();
            let other = Vector::zero(ctx, 1).unwrap();
            let combined = vector.concat(other).unwrap();
            assert_eq!(combined.get_element_val(1).unwrap().to_f64(), 3.0);
            assert_eq!(combined.zero_extend(5).unwrap().size().unwrap(), 5);
        });
    }
}
