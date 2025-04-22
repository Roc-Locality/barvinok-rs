use std::ops::Not;

use barvinok_sys::isl_bool;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Flag {
    True,
    False,
    Error,
}

impl Flag {
    pub fn from_isl_bool(stat: isl_bool) -> Self {
        match stat.cmp(&0) {
            std::cmp::Ordering::Greater => Flag::True,
            std::cmp::Ordering::Less => Flag::False,
            std::cmp::Ordering::Equal => Flag::Error,
        }
    }
}

impl From<bool> for Flag {
    fn from(value: bool) -> Self {
        if value { Flag::True } else { Flag::False }
    }
}

impl Not for Flag {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Flag::True => Flag::False,
            Flag::False => Flag::True,
            Flag::Error => Flag::Error,
        }
    }
}
