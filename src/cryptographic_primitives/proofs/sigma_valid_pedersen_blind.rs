/*
    Cryptography utilities

    Copyright 2018 by Kzen Networks

    This file is part of Cryptography utilities library
    (https://github.com/KZen-networks/cryptography-utils)

    Cryptography utilities is free software: you can redistribute
    it and/or modify it under the terms of the GNU General Public
    License as published by the Free Software Foundation, either
    version 3 of the License, or (at your option) any later version.

    @license GPL-3.0+ <https://github.com/KZen-networks/cryptography-utils/blob/master/LICENSE>
*/

// TODO: abstract for use with elliptic curves other than secp256k1
/// protocol for proving that Pedersen commitment c was constructed correctly which is the same as
/// proof of knowledge of (r) such that c = mG + rH.
/// witness: (r), statement: (c,m), The Relation R outputs 1 if c = mG + rH. The protocol:
/// 1: Prover chooses A = s*H for random s
/// prover calculates challenge e = H(G,H,c,A,m)
/// prover calculates z  = s + er,
/// prover sends pi = {e, m,A,c, z}
///
/// verifier checks that e*m*G* + zH  = A + ec
use BigInt;
use super::ProofError;
use arithmetic::traits::Converter;
use arithmetic::traits::Modulo;
use arithmetic::traits::Samplable;

use elliptic::curves::traits::*;

use cryptographic_primitives::hashing::hash_sha256::HSha256;
use cryptographic_primitives::hashing::traits::Hash;

use cryptographic_primitives::commitments::pedersen_commitment::pedersenCommitment;
use cryptographic_primitives::commitments::traits::Commitment;

use elliptic::curves::secp256_k1::Secp256k1Scalar;
use elliptic::curves::secp256_k1::Secp256k1Point;

#[derive(Clone, PartialEq, Debug)]
pub struct PedersenBlindingProof {
     e : Secp256k1Scalar,
     pub m : Secp256k1Scalar,
     A: Secp256k1Point,
     pub com: Secp256k1Point,
     z: Secp256k1Scalar,

}
pub trait ProvePederesenBlind {
    fn prove(m: &Secp256k1Scalar, r: &Secp256k1Scalar) -> PedersenBlindingProof;

    fn verify(proof: &PedersenBlindingProof) -> Result<(), ProofError>;
}

impl ProvePederesenBlind for PedersenBlindingProof {
    fn prove(m: &Secp256k1Scalar, r: &Secp256k1Scalar) -> PedersenBlindingProof {
        let h = Secp256k1Point::base_point2();
        let s: Secp256k1Scalar = ECScalar::new_random();
        let A = h.scalar_mul(&s.get_element());
        let com = pedersenCommitment::create_commitment_with_user_defined_randomness(&m.to_big_int(), &r.to_big_int());
        let G: Secp256k1Point = ECPoint::new();
        let challenge = HSha256::create_hash(vec![
            &G.get_x_coor_as_big_int(),
            &Secp256k1Point::base_point2().get_x_coor_as_big_int(),
            &com.get_x_coor_as_big_int(),
            &A.get_x_coor_as_big_int(),
            &m.to_big_int(),
        ]);
        let e: Secp256k1Scalar = ECScalar::from_big_int(&challenge);
        let er = e.mul(&r.get_element());
        let z = s.add(&er.get_element());
        PedersenBlindingProof{e, m:m.clone(), A, com, z}

    }

    fn verify(proof: &PedersenBlindingProof) -> Result<(), ProofError>{
        let g: Secp256k1Point = ECPoint::new();
        let h = Secp256k1Point::base_point2();
        let challenge = HSha256::create_hash(vec![
            &g.get_x_coor_as_big_int(),
            &h.get_x_coor_as_big_int(),
            &proof.com.get_x_coor_as_big_int(),
            &proof.A.get_x_coor_as_big_int(),
            &proof.m.to_big_int(),
        ]);
        let e: Secp256k1Scalar = ECScalar::from_big_int(&challenge);
        let zH = h.scalar_mul(&proof.z.get_element());
        let mG = g.scalar_mul(&proof.m.get_element());
        let emG = mG.scalar_mul(&e.get_element());
        let lhs = zH.add_point(&emG.get_element());
        let com_clone = proof.com.clone();
        let ecom = com_clone.scalar_mul(&e.get_element());
        let rhs = ecom.add_point(&proof.A.get_element());
        if lhs.get_element() == rhs.get_element() {
            Ok(())
        } else {
            Err(ProofError)
        }

    }
}

#[cfg(test)]
mod tests {
    use BigInt;
    use super::ProofError;
    use arithmetic::traits::Converter;
    use arithmetic::traits::Modulo;
    use arithmetic::traits::Samplable;

    use elliptic::curves::traits::*;

    use cryptographic_primitives::hashing::hash_sha256::HSha256;
    use cryptographic_primitives::hashing::traits::Hash;

    use cryptographic_primitives::commitments::pedersen_commitment::pedersenCommitment;
    use cryptographic_primitives::proofs::sigma_valid_pedersen_blind::*;

    use elliptic::curves::secp256_k1::Secp256k1Scalar;
    use elliptic::curves::secp256_k1::Secp256k1Point;

    #[test]
    fn test_pedersen_blind_proof() {
        let m: Secp256k1Scalar = ECScalar::new_random();
        let r: Secp256k1Scalar = ECScalar::new_random();
        let pedersen_proof = PedersenBlindingProof::prove(&m, &r);
        let verified = PedersenBlindingProof::verify(&pedersen_proof).expect("error pedersen blind");


    }

}