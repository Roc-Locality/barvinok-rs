use crate::{Context, impl_isl_handle};

impl_isl_handle!(Vector, vec);

impl<'a> Vector<'a> {
    pub fn new(ctx: &'a Context, dim: u32) -> Self {
        let handle = unsafe { barvinok_sys::isl_vec_alloc(ctx.0.as_ptr(), dim) };
        let handle = std::ptr::NonNull::new(handle).expect("Failed to allocate vector");
        Self {
            handle,
            marker: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_vector_creation() {
        let ctx = Context::new();
        let vector = Vector::new(&ctx, 5);
        println!("{:?}", vector);
    }
}
