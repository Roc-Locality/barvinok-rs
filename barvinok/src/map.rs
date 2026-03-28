use crate::aff::Affine;
use crate::list::List;
use crate::space::Space;
use crate::{DimType, constraint::Constraint, impl_isl_handle, isl_ctor};

impl_isl_handle!(Map, map);
impl_isl_handle!(BasicMap, basic_map);

include!(concat!(env!("OUT_DIR"), "/generated/basic_map.rs"));
include!(concat!(env!("OUT_DIR"), "/generated/map.rs"));

impl<'a> TryFrom<Constraint<'a>> for BasicMap<'a> {
    fn try_from(constraint: Constraint<'a>) -> Result<Self, Self::Error> {
        Self::from_constraint(constraint)
    }

    type Error = crate::Error;
}

impl<'a> TryFrom<Affine<'a>> for Map<'a> {
    fn try_from(aff: Affine<'a>) -> Result<Self, Self::Error> {
        Self::from_aff(aff)
    }

    type Error = crate::Error;
}

impl<'a> TryFrom<BasicMap<'a>> for Map<'a> {
    fn try_from(basic_map: BasicMap<'a>) -> Result<Self, Self::Error> {
        Self::from_basic_map(basic_map)
    }
    type Error = crate::Error;
}

impl<'a> BasicMap<'a> {
    isl_ctor!(from_affine_list, isl_basic_map_from_aff_list, domain_space: Space<'a>, [managed] aff: List<'a, Affine<'a>>);
}

#[cfg(test)]
mod tests {
    use crate::{
        Context, DimType,
        constraint::Constraint,
        local_space::LocalSpace,
        map::{BasicMap, Map},
        space::Space,
    };

    #[test]
    fn test_from_constraints() {
        let context = Context::new();
        context.scope(|context| {
            let space = Space::new(context, 1, 2, 2).unwrap();
            let local_space = LocalSpace::try_from(space).unwrap();
            let mut constraint = Constraint::new_inequality(local_space).unwrap();
            constraint = constraint.set_constant_si(5).unwrap();
            constraint = constraint.set_coefficient_si(DimType::Param, 0, 3).unwrap();
            let basic_set = BasicMap::try_from(constraint).unwrap();
            println!("Basic Map: {:?}", basic_set);
        });
    }

    #[test]
    fn test_lex_lt_on_space() -> anyhow::Result<()> {
        let context = Context::new();
        context.scope(|context| {
            let space = Space::set(context, 2, 2).unwrap();
            let basic_map = Map::lex_lt(space)?;
            println!("Lexicographically less than map: {:?}", basic_map);
            Ok(())
        })
    }
}
