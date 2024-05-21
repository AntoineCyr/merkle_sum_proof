#![allow(clippy::derive_hash_xor_eq)]
#![allow(clippy::too_many_arguments)]
use crate::constants::C_STR;
use ff::{self, *};
use once_cell::sync::Lazy;
use std::ops::AddAssign;

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
pub struct Fr([u64; 4]);

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
    //create function to get all proof of inclusion inputs
    #[test]
    fn it_works() {
        //leaf_1: Leaf { id: "11672136", id_hash: Fr(0x0000000000000000000000000000000000000000000000003309c891ce14a103), value: 10 }
        //leaf_2: Leaf { id: "10566265", id_hash: Fr(0x000000000000000000000000000000000000000000000000c0cd4e53cd09276f), value: 11 }
        //root 0x0467bba3be1311be62439b3e7cb08695cfefc186add485bdf89bd0bb30e9bb4f

        let arr = vec![
            Fr::from_str_vartime("3677691099277992195").unwrap(),
            Fr::from_str_vartime("10").unwrap(),
            Fr::from_str_vartime("13892846547337029487").unwrap(),
            Fr::from_str_vartime("11").unwrap(),
        ];
        println!("arr: {:?}", arr);
        let k = Fr::from_str_vartime("0").unwrap();
        let ms = MimcSponge::default();
        let res = ms.multi_hash(&arr, k, 1);
        println!("res:  {:?}", res[0]);
    }
}
