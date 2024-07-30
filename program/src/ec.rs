use ark_bn254::{
    g1::{self, G1Affine},
    Fq, Fr, G1Projective as G1,
};
use ark_ec::{
    hashing::{
        map_to_curve_hasher::{MapToCurve, MapToCurveBasedHasher},
        HashToCurve, HashToCurveError,
    },
    short_weierstrass::{Affine, Projective, SWCurveConfig},
};
use ark_ff::{field_hashers::DefaultFieldHasher, BigInteger, Field, MontFp, PrimeField};
use ark_std::{ops::Mul, Zero};
use sha3::Keccak256;
use std::marker::PhantomData;

pub trait SVDWConfig: SWCurveConfig {
    const Z: Self::BaseField;
    // g(Z), g(x) = x^3 + 3
    const C1: Self::BaseField;
    // -Z / 2
    const C2: Self::BaseField;
    // sqrt(-g(Z) * (3 * Z^2 + 4 * A)) and sgn0(C3) == 0
    const C3: Self::BaseField;
    // -4 * -g(Z) / (3 * Z^2 + 4 * A)
    const C4: Self::BaseField;
    // (N - 1) / 2
    // const C5: Self::BaseField;
}

impl SVDWConfig for g1::Config {
    const Z: Fq = MontFp!("1");
    const C1: Fq = MontFp!("4");
    const C2: Fq =
        MontFp!("10944121435919637611123202872628637544348155578648911831344518947322613104291");
    const C3: Fq = MontFp!("8815841940592487685674414971303048083897117035520822607866");
    const C4: Fq =
        MontFp!("7296080957279758407415468581752425029565437052432607887563012631548408736189");
}

pub struct SVDWMap<P: SVDWConfig>(PhantomData<fn() -> P>);

/// Trait defining a parity method on the Field elements based on [\[1\]] Section 4.1
///
/// - [\[1\]] <https://datatracker.ietf.org/doc/draft-irtf-cfrg-hash-to-curve/>
pub(crate) fn parity<F: Field>(element: &F) -> bool {
    element
        .to_base_prime_field_elements()
        .find(|&x| !x.is_zero())
        .map_or(false, |x| x.into_bigint().is_odd())
}

impl<P: SVDWConfig> MapToCurve<Projective<P>> for SVDWMap<P> {
    /// Constructs a new map if `P` represents a valid map.
    fn new() -> Result<Self, HashToCurveError> {
        if parity(&P::C3) {
            return Err(HashToCurveError::MapToCurveError(
                "sgn0(C3) != 0".to_string(),
            ));
        }
        Ok(SVDWMap(PhantomData))
    }

    fn map_to_curve(&self, u: P::BaseField) -> Result<Affine<P>, HashToCurveError> {
        let mut tv1 = u.square() * P::C1;
        let tv2 = P::BaseField::ONE + tv1;
        tv1 = P::BaseField::ONE - tv1;
        let tv3 = (tv1 * tv2)
            .inverse()
            .ok_or(HashToCurveError::MapToCurveError(
                "inv(tv1 * tv2) failed".to_string(),
            ))?;
        let mut tv4 = P::C3;
        if parity(&tv4) {
            tv4 = -tv4;
        }
        let tv5 = u * tv1 * tv3 * tv4;
        let tv6 = P::C4;
        let x1 = P::C2 - tv5;
        let gx1 = x1 * x1 * x1 + P::COEFF_B;
        let x2 = P::C2 + tv5;
        let gx2 = x2 * x2 * x2 + P::COEFF_B;
        let x3 = P::Z + tv6 * (tv2.square() * tv3).square();
        let gx3 = x3 * x3 * x3 + P::COEFF_B;
        let x;
        let mut y;
        if gx1.legendre().is_qr() {
            x = x1;
            y = gx1.sqrt().ok_or(HashToCurveError::MapToCurveError(
                "sqrt(g(x1)) failed".to_string(),
            ))?;
        } else if gx2.legendre().is_qr() {
            x = x2;
            y = gx2.sqrt().ok_or(HashToCurveError::MapToCurveError(
                "sqrt(g(x2)) failed".to_string(),
            ))?;
        } else {
            x = x3;
            y = gx3.sqrt().ok_or(HashToCurveError::MapToCurveError(
                "sqrt(g(x3)) failed".to_string(),
            ))?;
        }
        if parity(&u) != parity(&y) {
            y = -y;
        }

        Ok(Affine::<P>::new_unchecked(x, y))
    }
}

pub fn get_pedersen_base(i: usize) -> G1Affine {
    let curve_hasher = MapToCurveBasedHasher::<
        Projective<g1::Config>,
        DefaultFieldHasher<Keccak256, 128>,
        SVDWMap<g1::Config>,
    >::new(&[] /* DST */)
    .unwrap();
    curve_hasher.hash(&(i as u64).to_be_bytes()).unwrap()
}

pub fn commit_single(value: Fr, i: usize) -> G1 {
    get_pedersen_base(i).mul(value)
}
