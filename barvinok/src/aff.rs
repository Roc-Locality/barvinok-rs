use crate::DimType;

include!(concat!(env!("OUT_DIR"), "/generated/aff.rs"));

impl<'a> std::ops::Neg for Affine<'a> {
    type Output = Affine<'a>;
    fn neg(self) -> Self::Output {
        self.checked_neg().unwrap()
    }
}

impl<'a> std::ops::Add for Affine<'a> {
    type Output = Affine<'a>;
    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs).unwrap()
    }
}

impl<'a> std::ops::Sub for Affine<'a> {
    type Output = Affine<'a>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs).unwrap()
    }
}

impl<'a> std::ops::Mul for Affine<'a> {
    type Output = Affine<'a>;
    fn mul(self, rhs: Self) -> Self::Output {
        self.checked_mul(rhs).unwrap()
    }
}
impl<'a> std::ops::Div for Affine<'a> {
    type Output = Affine<'a>;
    fn div(self, rhs: Self) -> Self::Output {
        self.checked_div(rhs).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;
    use crate::local_space::LocalSpace;
    use crate::space::Space;

    #[test]
    fn test_aff() -> anyhow::Result<()> {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 2)?;
            let aff = Affine::zero_on_domain_space(space)?;
            println!("Affine: {:?}", aff);
            Ok(())
        })
    }

    #[test]
    fn test_aff_with_name() -> anyhow::Result<()> {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 2)?;
            let aff = Affine::zero_on_domain_space(space)?
                .set_dim_name(DimType::Param, 0, "N")?
                .set_dim_name(DimType::Param, 1, "M")?
                .set_dim_name(DimType::In, 0, "i")?
                .set_dim_name(DimType::In, 1, "j")?;
            println!("Affine: {:?}", aff);
            Ok(())
        })
    }

    #[test]
    fn test_aff_binary() -> anyhow::Result<()> {
        let ctx = Context::new();
        ctx.scope(|ctx| {
            let space = Space::set(ctx, 2, 2)?;
            let local_space = LocalSpace::try_from(space).unwrap();
            let aff1 = Affine::var_on_domain(local_space.clone(), DimType::Param, 0)?;
            let aff2 = Affine::var_on_domain(local_space.clone(), DimType::Out, 1)?;
            let aff3 = Affine::zero_on_domain(local_space)?;
            let aff4 = aff1 + aff2;
            let aff5 = aff4 + aff3;
            println!("Affine: {:?}", aff5);
            Ok(())
        })
    }
}
