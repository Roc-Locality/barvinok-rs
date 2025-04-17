use std::ptr::NonNull;

#[repr(transparent)]
pub struct QuasiPolynomial<'a> {
    handle: NonNull<barvinok_sys::isl_pw_qpolynomial>,
    marker: std::marker::PhantomData<&'a ()>,
}
