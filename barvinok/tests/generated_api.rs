use anyhow::Result;
use barvinok::{
    Context, DimType,
    constraint::Constraint,
    ident::Ident,
    local_space::LocalSpace,
    mat::Matrix,
    multi_aff::MultiAffine,
    multi_pw_aff::MultiPiecewiseAffine,
    point::Point,
    polynomial::{PiecewiseQuasiPolynomial, QuasiPolynomial},
    pw_aff::PiecewiseAffine,
    pw_multi_aff::PiecewiseMultiAffine,
    set::BasicSet,
    space::Space,
    union_map::UnionMap,
    union_pw_aff::UnionPiecewiseAffine,
    union_pw_multi_aff::UnionPiecewiseMultiAffine,
    union_set::UnionSet,
    value::Value,
    vec::Vector,
};

#[test]
fn generated_value_and_vector_surface() -> Result<()> {
    let context = Context::new();
    context.scope(|ctx| -> Result<()> {
        let parsed = Value::from_str(ctx, "42/5")?;
        assert_eq!(parsed.get_den_val()?.to_f64(), 5.0);

        let gcd = Value::int_from_si(ctx, 42)?.gcd(Value::int_from_si(ctx, 7)?)?;
        assert_eq!(gcd.to_f64(), 7.0);
        assert!(Value::one(ctx)?.is_pos()?);

        let vector = Vector::new(ctx, 2)?.set_element_si(1, 3)?;
        assert_eq!(vector.get_element_val(1)?.to_f64(), 3.0);

        let combined = vector.concat(Vector::zero(ctx, 1)?)?;
        assert_eq!(combined.zero_extend(5)?.size()?, 5);
        Ok(())
    })
}

#[test]
fn generated_space_local_space_and_affine_surface() -> Result<()> {
    let context = Context::new();
    context.scope(|ctx| -> Result<()> {
        let param_id = Ident::new(ctx, "p")?;
        let param_space = Space::params(ctx, 1)?.set_dim_id(DimType::Param, 0, param_id)?;
        assert!(param_space.has_dim_id(DimType::Param, 0)?);

        let set_space = Space::set(ctx, 0, 2)?
            .set_tuple_name(DimType::In, "S")?
            .set_dim_name(DimType::Out, 0, "x")?;
        assert!(set_space.has_tuple_name(DimType::In)?);
        assert_eq!(set_space.get_tuple_name(DimType::In)?, Some("S"));
        assert_eq!(set_space.get_dim_name(DimType::Out, 0)?, Some("x"));

        let local = LocalSpace::from_space(set_space.clone())?;
        assert_eq!(local.dim(DimType::Param)?, 0);
        assert_eq!(local.dim(DimType::Out)?, 2);

        let params_local = LocalSpace::from_space(Space::params(ctx, 1)?)?;
        let set_local = params_local.clone().set_from_params()?;
        assert!(set_local.get_space()?.is_set()?);
        assert!(
            set_local
                .clone()
                .intersect(set_local.clone())?
                .is_equal(&set_local)?
        );

        let affine_space = Space::set(ctx, 0, 2)?;
        let affine = affine_space
            .clone()
            .zero_aff_on_domain()?
            .add_constant_si(9)?;
        assert_eq!(affine.eval(Point::zero(affine_space)?)?.to_f64(), 9.0);
        Ok(())
    })
}

#[test]
fn generated_set_map_and_constraint_surface() -> Result<()> {
    let context = Context::new();
    context.scope(|ctx| -> Result<()> {
        let set_local = LocalSpace::from_space(Space::set(ctx, 0, 1)?)?;
        let constraint = Constraint::new_inequality(set_local)?
            .set_coefficient_si(DimType::Out, 0, 1)?
            .set_constant_si(0)?;
        let basic_set = BasicSet::from_constraint(constraint)?;
        assert_eq!(basic_set.n_constraint()?, 1);
        assert!(!basic_set.clone().to_set()?.is_empty()?);

        let map = Space::set(ctx, 0, 1)?
            .map_from_set()?
            .universe_map()?
            .set_tuple_name(DimType::In, "D")?
            .set_tuple_name(DimType::Out, "R")?
            .coalesce()?;
        let basic_map = map.clone().affine_hull()?;
        assert_eq!(map.get_tuple_name(DimType::In)?, Some("D"));
        assert_eq!(map.get_tuple_name(DimType::Out)?, Some("R"));
        assert!(!map.is_empty()?);
        assert!(!basic_map.is_empty()?);
        Ok(())
    })
}

#[test]
fn generated_matrix_and_polynomial_surface() -> Result<()> {
    let context = Context::new();
    context.scope(|ctx| -> Result<()> {
        let vector = Vector::zero(ctx, 2)?
            .set_element_si(0, 1)?
            .set_element_si(1, 3)?;
        let identity = Matrix::identity(ctx, 2)?;
        assert_eq!(identity.rows()?, 2);
        assert_eq!(identity.cols()?, 2);
        assert!(
            identity
                .clone()
                .vec_product(vector.clone())?
                .is_equal(&vector)?
        );

        let space = Space::set(ctx, 0, 1)?;
        let qpoly =
            QuasiPolynomial::from_aff(space.clone().zero_aff_on_domain()?.add_constant_si(4)?)?;
        assert!(qpoly.isa_aff()?);

        let pw = PiecewiseQuasiPolynomial::from_qpolynomial(qpoly)?.coalesce()?;
        assert!(pw.is_qpolynomial()?);
        assert_eq!(pw.num_pieces()?, 1);
        assert_eq!(pw.eval(Point::zero(space)?)?.to_f64(), 4.0);
        Ok(())
    })
}

    #[test]
    fn generated_piecewise_multi_and_union_surface() -> Result<()> {
        let context = Context::new();
        context.scope(|ctx| -> Result<()> {
            let set_space = Space::set(ctx, 0, 1)?;
            let set = barvinok::set::Set::universe(set_space.clone())?;
            let pw_aff = PiecewiseAffine::val_on_domain(set.clone(), Value::one(ctx)?)?;
            assert!(pw_aff.clone().domain()?.is_equal(&set)?);

            let multi_pw_aff = MultiPiecewiseAffine::from_pw_aff(pw_aff.clone())?;
            assert!(multi_pw_aff.clone().as_map()?.domain()?.is_equal(&set)?);

            let pw_multi_aff = PiecewiseMultiAffine::from_pw_aff(pw_aff.clone())?;
            assert!(pw_multi_aff.clone().domain()?.is_equal(&set)?);

            let multi_aff = MultiAffine::domain_map(set_space.clone().map_from_set()?)?;
            let pw_multi_from_multi = multi_aff.clone().to_pw_multi_aff()?;
            let multi_pw_from_multi = multi_aff.clone().to_multi_pw_aff()?;
            assert!(
                multi_aff
                .clone()
                .as_map()?
                .is_equal(&multi_pw_from_multi.clone().as_map()?)?
        );
        assert!(
            pw_multi_from_multi
                .clone()
                .as_map()?
                .is_equal(&multi_pw_from_multi.as_map()?)?
            );

            let union_set = UnionSet::from_set(set.clone())?;
            assert_eq!(union_set.n_set()?, 1);
            assert!(union_set.clone().as_set()?.is_equal(&set)?);

            let union_pw_aff = UnionPiecewiseAffine::from_pw_aff(pw_aff)?;
            assert!(union_pw_aff.clone().domain()?.is_equal(&union_set)?);

            let union_pw_multi_aff =
                UnionPiecewiseMultiAffine::from_union_pw_aff(union_pw_aff.clone())?;
            let union_map = union_pw_multi_aff.clone().as_union_map()?;
            assert!(union_map.clone().domain()?.is_equal(&union_set)?);

            let roundtrip_union_pw_multi_aff = union_map.clone().as_union_pw_multi_aff()?;
            assert!(roundtrip_union_pw_multi_aff.domain()?.is_equal(&union_set)?);

            let multi_union_pw_aff = union_map.as_multi_union_pw_aff()?;
            assert!(multi_union_pw_aff.domain()?.is_equal(&union_set)?);

            let union_map_from_map = UnionMap::from_map(pw_multi_aff.as_map()?)?;
            assert!(union_map_from_map.domain()?.as_set()?.is_equal(&set)?);
            Ok(())
        })
}
