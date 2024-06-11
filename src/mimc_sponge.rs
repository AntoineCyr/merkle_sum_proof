#![allow(clippy::derive_hash_xor_eq)]
#![allow(clippy::too_many_arguments)]
use crate::constants::C_STR;
use ff::{self, *};
use num::{BigInt, Num};
use once_cell::sync::Lazy;
use regex::Regex;
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

const DEFAULT_CONSTS_LEN: usize = C_STR.len();
static DEFAULT_CONSTS: Lazy<[Fr; DEFAULT_CONSTS_LEN]> = Lazy::new(|| {
    C_STR
        .iter()
        .map(|s| Fr::from_str_vartime(s).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
});

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
            t = Fr::from_str_vartime("0").unwrap();

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

    /// Takes &slice of Fr elements, key and num_outputs
    pub fn multi_hash(&self, arr: &[Fr], key: Fr, num_outputs: usize) -> Vec<Fr> {
        let mut r = Fr::from_str_vartime("0").unwrap();
        let mut c = Fr::from_str_vartime("0").unwrap();

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
    fn it_works() {
        let arr = vec![
            Fr::from_str_vartime("11672136").unwrap(),
            Fr::from_str_vartime("10").unwrap(),
            Fr::from_str_vartime("10566265").unwrap(),
            Fr::from_str_vartime("11").unwrap(),
        ];
        println!("arr: {:?}", arr);
        let k = Fr::from_str_vartime("0").unwrap();
        let ms = MimcSponge::default();
        let res = ms.multi_hash(&arr, k, 1);
        println!("res: {}", res[0].to_string());
    }
}
