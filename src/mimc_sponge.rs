use crate::constants::C_STR;
use ff::{self, *};
use once_cell::sync::Lazy;
use std::fmt;
use std::ops::AddAssign;

#[derive(PrimeField)]
//Modulus for circom
//#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
//Modulus for nova folding
#[PrimeFieldModulus = "28948022309329048855892746252171976963363056481941647379679742748393362948097"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
pub struct Fr([u64; 4]);

impl fmt::Display for Fr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut format = format!("{:?}", self);
        format = format.replace("Fr(", "");
        format = format.replace(")", "");
        write!(f, "{}", format)
    }
}

impl Fr {
    /// Returns a zero-valued Fr element
    pub const fn zero() -> Self {
        Fr([0, 0, 0, 0])
    }
}

const DEFAULT_CONSTS_LEN: usize = C_STR.len();
static DEFAULT_CONSTS: Lazy<[Fr; DEFAULT_CONSTS_LEN]> = Lazy::new(|| {
    C_STR
        .iter()
        .map(|s| Fr::from_str_vartime(s).expect("Valid constant from C_STR"))
        .collect::<Vec<_>>()
        .try_into()
        .expect("Correct number of constants")
});

/// MiMC sponge hash function implementation
pub struct MimcSponge {
    constants: [Fr; DEFAULT_CONSTS_LEN],
}

impl Default for MimcSponge {
    fn default() -> Self {
        Self {
            constants: *DEFAULT_CONSTS,
        }
    }
}

impl MimcSponge {
    fn hash(&self, mut xl: Fr, mut xr: Fr, k: Fr) -> (Fr, Fr) {
        let mut t;
        let mut xr_tmp;
        let last_index = self.constants.len() - 1;

        for (i, c) in self.constants.iter().enumerate() {
            t = Fr::zero();

            t.add_assign(&xl);
            t.add_assign(&k);

            if i > 0 {
                t.add_assign(c);
            }

            t = t.pow([5u64]);

            xr_tmp = xr;
            xr_tmp.add_assign(&t);

            if i < last_index {
                xr = xl;
                xl = xr_tmp;
            } else {
                xr = xr_tmp
            }
        }

        (xl, xr)
    }

    pub fn multi_hash(&self, arr: &[Fr], key: Fr, num_outputs: usize) -> Vec<Fr> {
        let mut r = Fr::zero();
        let mut c = Fr::zero();

        for elem in arr {
            r.add_assign(elem);
            let s: (Fr, Fr) = self.hash(r, c, key);
            (r, c) = s;
        }

        let mut out = Vec::with_capacity(num_outputs);
        out.push(r);

        for _ in 1..num_outputs {
            let s = self.hash(r, c, key);
            (r, c) = s;
            out.push(r);
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mimc_multi_hash_computation() {
        let arr = vec![
            Fr::from_str_vartime("11672136").expect("Valid test value"),
            Fr::from_str_vartime("10").expect("Valid test value"),
            Fr::from_str_vartime("10566265").expect("Valid test value"),
            Fr::from_str_vartime("11").expect("Valid test value"),
        ];
        let k = Fr::zero();
        let ms = MimcSponge::default();
        let res = ms.multi_hash(&arr, k, 1);

        // Ensure hash computation returns non-zero result
        assert!(!res.is_empty());
        assert_ne!(res[0], Fr::zero());
    }
}
