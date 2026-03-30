use barvinok_sys::{isl_bool, isl_size};

pub(crate) fn isl_bool_to_optional_bool(b: isl_bool) -> Option<bool> {
    match b.cmp(&0) {
        std::cmp::Ordering::Less => None,
        std::cmp::Ordering::Equal => Some(false),
        std::cmp::Ordering::Greater => Some(true),
    }
}

pub(crate) fn isl_size_to_optional_u32(s: isl_size) -> Option<u32> {
    if s < 0 { None } else { Some(s as u32) }
}
