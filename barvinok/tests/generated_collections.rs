use anyhow::Result;
use barvinok::{
    Context, DimType,
    constraint::Constraint,
    ident::Ident,
    list::{BasicSetList, ConstraintList, IdentList, SetList, ValueList},
    local_space::LocalSpace,
    multi::{DomainMulti, Multi, MultiList, MutableMulti},
    multi_aff::MultiAffine,
    multi_id::MultiId,
    multi_pw_aff::MultiPiecewiseAffine,
    multi_val::MultiValue,
    point::Point,
    pw_aff::PiecewiseAffine,
    pw_multi_aff::PiecewiseMultiAffine,
    set::{BasicSet, Set},
    space::Space,
    value::Value,
};

#[test]
fn concrete_list_alias_surface() -> Result<()> {
    let context = Context::new();
    context.scope(|ctx| -> Result<()> {
        let mut values = ValueList::new(ctx, 2);
        let one = Value::one(ctx)?;
        let two = Value::int_from_si(ctx, 2)?;
        values.push(one.clone());
        values.push(two.clone());
        assert_eq!(values.len(), 2);
        assert_eq!(values.get(0).unwrap().to_f64(), 1.0);
        assert_eq!(values.get(1).unwrap().to_f64(), 2.0);

        values.set(1, one.clone() + one.clone());
        let items = values
            .clone()
            .iter()
            .map(|value| value.to_f64())
            .collect::<Vec<_>>();
        assert_eq!(items, vec![1.0, 2.0]);

        let singleton = ValueList::new_singleton(two);
        assert_eq!(singleton.len(), 1);
        assert_eq!(singleton.get(0).unwrap().to_f64(), 2.0);

        let space = Space::set(ctx, 0, 1)?;
        let universe = Set::universe(space.clone())?;
        let empty = Set::empty(space.clone())?;
        let mut sets = SetList::new(ctx, 2);
        sets.push(universe.clone());
        sets.push(empty);
        assert!(sets.union().is_equal(&universe)?);

        let basic_universe = BasicSet::universe(space.clone())?;
        let basic_empty = BasicSet::empty(space)?;
        let mut basic_sets = BasicSetList::new(ctx, 2);
        basic_sets.push(basic_universe);
        basic_sets.push(basic_empty);
        assert!(basic_sets.intersect().is_empty()?);
        Ok(())
    })
}

#[test]
fn constraint_list_alias_roundtrip() -> Result<()> {
    let context = Context::new();
    context.scope(|ctx| -> Result<()> {
        let local_space = LocalSpace::from_space(Space::set(ctx, 0, 1)?)?;
        let constraint = Constraint::new_inequality(local_space)?
            .set_coefficient_si(DimType::Out, 0, 1)?
            .set_constant_si(0)?;
        let basic_set = BasicSet::from_constraint(constraint)?;
        let constraints: ConstraintList<'_> = basic_set.get_constraints()?;
        assert_eq!(constraints.len(), 1);
        assert!(!constraints.get(0).unwrap().is_equality()?);
        Ok(())
    })
}

#[test]
fn high_level_multi_affine_api() -> Result<()> {
    let context = Context::new();
    context.scope(|ctx| -> Result<()> {
        let domain_space = Space::set(ctx, 0, 1)?;
        let point = Point::zero(domain_space.clone())?;
        let first = MultiAffine::from_aff(
            domain_space
                .clone()
                .zero_aff_on_domain()?
                .add_constant_si(1)?,
        )?;
        let second = MultiAffine::from_aff(
            domain_space
                .clone()
                .zero_aff_on_domain()?
                .add_constant_si(2)?,
        )?;
        let multi = first.flat_range_product(second)?;
        let rebuilt = MultiAffine::from_list(multi.space()?, multi.list()?)?;

        assert_eq!(rebuilt.component_count()?, 2);
        assert!(rebuilt.space()?.is_map()?);
        assert!(rebuilt.domain_space()?.is_set()?);
        assert_eq!(
            rebuilt
                .first_component()?
                .unwrap()
                .eval(point.clone())?
                .to_f64(),
            1.0
        );
        assert_eq!(rebuilt.components()?.len(), 2);
        assert_eq!(rebuilt.list()?.len(), 2);

        let replaced = rebuilt.set_component(
            1,
            domain_space
                .clone()
                .zero_aff_on_domain()?
                .add_constant_si(3)?,
        )?;
        let second = replaced.component(1)?.eval(point)?.to_f64();
        assert_eq!(second, 3.0);
        Ok(())
    })
}

#[test]
fn high_level_multi_value_and_id_api() -> Result<()> {
    let context = Context::new();
    context.scope(|ctx| -> Result<()> {
        let space = Space::set(ctx, 0, 2)?;

        let mut values = ValueList::new(ctx, 2);
        values.push(Value::int_from_si(ctx, 5)?);
        values.push(Value::int_from_si(ctx, 8)?);
        let multi_val = MultiValue::from_list(space.clone(), values)?;
        assert_eq!(multi_val.component_count()?, 2);
        assert_eq!(multi_val.first_component()?.unwrap().to_f64(), 5.0);
        assert_eq!(multi_val.last_component()?.unwrap().to_f64(), 8.0);
        assert_eq!(multi_val.list()?.len(), 2);
        let updated = multi_val.set_component(0, Value::one(ctx)?)?;
        assert_eq!(updated.component(0)?.to_f64(), 1.0);

        let mut ids = IdentList::new(ctx, 2);
        ids.push(Ident::new(ctx, "i")?);
        ids.push(Ident::new(ctx, "j")?);
        let multi_id = MultiId::from_list(space, ids)?;
        assert_eq!(multi_id.component_count()?, 2);
        assert_eq!(multi_id.first_component()?.unwrap().name()?, "i");
        assert_eq!(multi_id.last_component()?.unwrap().name()?, "j");
        assert_eq!(multi_id.list()?.len(), 2);
        Ok(())
    })
}

#[test]
fn high_level_piecewise_multi_api() -> Result<()> {
    let context = Context::new();
    context.scope(|ctx| -> Result<()> {
        let space = Space::set(ctx, 0, 1)?;
        let domain = Set::universe(space.clone())?;
        let point = Point::zero(space.clone())?;
        let first = MultiPiecewiseAffine::from_pw_aff(PiecewiseAffine::val_on_domain(
            domain.clone(),
            Value::int_from_si(ctx, 4)?,
        )?)?;
        let second = MultiPiecewiseAffine::from_pw_aff(PiecewiseAffine::val_on_domain(
            domain.clone(),
            Value::int_from_si(ctx, 7)?,
        )?)?;
        let multi_pw = first.flat_range_product(second)?;
        let rebuilt = MultiPiecewiseAffine::from_list(multi_pw.space()?, multi_pw.list()?)?;

        assert_eq!(rebuilt.component_count()?, 2);
        assert!(rebuilt.domain_space()?.is_set()?);
        assert_eq!(rebuilt.list()?.len(), 2);
        let changed = rebuilt.set_component(
            0,
            PiecewiseAffine::val_on_domain(domain.clone(), Value::int_from_si(ctx, 9)?)?,
        )?;
        assert_eq!(changed.component(0)?.eval(point.clone())?.to_f64(), 9.0);

        let pw_multi = PiecewiseMultiAffine::from_multi_pw_aff(changed)?;
        assert_eq!(pw_multi.component_count()?, 2);
        assert!(pw_multi.domain_space()?.is_set()?);
        assert_eq!(
            pw_multi.first_component()?.unwrap().eval(point)?.to_f64(),
            9.0
        );
        Ok(())
    })
}
