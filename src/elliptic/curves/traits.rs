/*
    This file is part of Curv library
    Copyright 2018 by Kzen Networks
    (https://github.com/KZen-networks/curv)
    License MIT: <https://github.com/KZen-networks/curv/blob/master/LICENSE>
*/

use std::fmt;

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::BigInt;

/// Elliptic curve implementation
///
/// Refers to according implementation of [ECPoint] and [ECScalar].
pub trait Curve {
    type Point: ECPoint<Scalar = Self::Scalar>;
    type Scalar: ECScalar;

    /// Canonical name for this curve
    const CURVE_NAME: &'static str;
}

/// Scalar value modulus [curve order](Self::curve_order)
///
/// ## Note
/// This is a low-level trait, you should not use it directly. See wrappers [Point], [PointZ],
/// [Scalar], [ScalarZ].
///
/// [Point]: super::wrappers::Point
/// [PointZ]: super::wrappers::PointZ
/// [Scalar]: super::wrappers::Scalar
/// [ScalarZ]: super::wrappers::ScalarZ
///
/// Trait exposes various methods to manipulate scalars. Scalar can be zero. Scalar must zeroize its
/// value on drop.
pub trait ECScalar: Clone + PartialEq + fmt::Debug + 'static {
    /// Underlying scalar type that can be retrieved in case of missing methods in this trait
    type Underlying;

    /// Samples a random scalar
    fn random() -> Self;

    /// Constructs a zero scalar
    fn zero() -> Self;
    /// Checks if the scalar equals to zero
    fn is_zero(&self) -> bool;

    /// Constructs a scalar `n % curve_order`
    fn from_bigint(n: &BigInt) -> Self;
    /// Converts a scalar to BigInt
    fn to_bigint(&self) -> BigInt;

    /// Calculates `(self + other) mod curve_order`
    fn add(&self, other: &Self) -> Self;
    /// Calculates `(self * other) mod curve_order`
    fn mul(&self, other: &Self) -> Self;
    /// Calculates `(self - other) mod curve_order`
    fn sub(&self, other: &Self) -> Self;
    /// Calculates `-self mod curve_order`
    fn neg(&self) -> Self;
    /// Calculates `self^-1 (mod curve_order)`, returns None if self equals to zero
    fn invert(&self) -> Option<Self>;
    /// Calculates `(self + other) mod curve_order`, and assigns result to `self`
    fn add_assign(&mut self, other: &Self) {
        *self = self.add(other)
    }
    /// Calculates `(self * other) mod curve_order`, and assigns result to `self`
    fn mul_assign(&mut self, other: &Self) {
        *self = self.mul(other)
    }
    /// Calculates `(self - other) mod curve_order`, and assigns result to `self`
    fn sub_assign(&mut self, other: &Self) {
        *self = self.sub(other)
    }
    /// Calculates `-self mod curve_order`, and assigns result to `self`
    fn neg_assign(&mut self) {
        *self = self.neg()
    }

    fn curve_order() -> &'static BigInt;

    /// Returns a reference to underlying scalar value
    fn underlying_ref(&self) -> &Self::Underlying;
    /// Returns a mutable reference to underlying scalar value
    fn underlying_mut(&mut self) -> &mut Self::Underlying;
    /// Constructs a scalar from underlying value
    fn from_underlying(u: Self::Underlying) -> Self;
}

/// Point on elliptic curve
///
/// ## Note
/// This is a low-level trait, you should not use it directly. See [Point], [PointZ], [Scalar],
/// [ScalarZ].
///
/// [Point]: super::wrappers::Point
/// [PointZ]: super::wrappers::PointZ
/// [Scalar]: super::wrappers::Scalar
/// [ScalarZ]: super::wrappers::ScalarZ
///
/// Trait exposes various methods that make elliptic curve arithmetic. The point can
/// be [zero](ECPoint::zero). Unlike [ECScalar], ECPoint isn't required to zeroize its value on drop,
/// but it implements [Zeroize] trait so you can force zeroizing policy on your own.
pub trait ECPoint: Zeroize + Clone + PartialEq + fmt::Debug + 'static {
    /// Scalar value the point can be multiplied at
    type Scalar: ECScalar;
    /// Underlying curve implementation that can be retrieved in case of missing methods in this trait
    type Underlying;

    /// Zero point
    ///
    /// Zero point is usually denoted as O. It's curve neutral element, i.e. `forall A. A + O = A`.
    /// Weierstrass and Montgomery curves employ special "point at infinity" to add neutral elements,
    /// such points don't have coordinates (i.e. [from_coords], [x_coord], [y_coord] return `None`).
    /// Edwards curves' neutral element has coordinates.
    ///
    /// [from_coords]: Self::from_coords
    /// [x_coord]: Self::x_coord
    /// [y_coord]: Self::y_coord
    fn zero() -> Self;

    /// Returns `true` if point is a neutral element
    fn is_zero(&self) -> bool;

    /// Curve generator
    ///
    /// Returns a static reference at actual value because in most cases reference value is fine.
    /// Use `.clone()` if you need to take it by value, i.e. `ECPoint::generator().clone()`
    fn generator() -> &'static Self;
    /// Curve second generator
    ///
    /// We provide an alternative generator value and prove that it was picked randomly
    fn base_point2() -> &'static Self;

    /// Constructs a curve point from its coordinates
    ///
    /// Returns error if x, y are not on curve
    fn from_coords(x: &BigInt, y: &BigInt) -> Result<Self, NotOnCurve>;
    /// Returns `x` coordinate of the point, or `None` if point is at infinity
    fn x_coord(&self) -> Option<BigInt>;
    /// Returns `y` coordinate of the point, or `None` if point is at infinity
    fn y_coord(&self) -> Option<BigInt>;
    /// Returns point coordinates (`x` and `y`), or `None` if point is at infinity
    fn coords(&self) -> Option<PointCoords>;

    /// Serializes point into bytes either in compressed or uncompressed form
    ///
    /// Returns None if point doesn't have coordinates, ie. it is "at infinity". If point isn't
    /// at infinity, serialize always succeeds.
    fn serialize(&self, compressed: bool) -> Option<Vec<u8>>;
    /// Deserializes point from bytes
    ///
    /// Whether point in compressed or uncompressed form will be deducted from its size
    fn deserialize(bytes: &[u8]) -> Result<Self, DeserializationError>;

    /// Multiplies the point at scalar value
    fn scalar_mul(&self, scalar: &Self::Scalar) -> Self;
    /// Adds two points
    fn add_point(&self, other: &Self) -> Self;
    /// Substrates `other` from `self`
    fn sub_point(&self, other: &Self) -> Self;
    /// Negates point
    fn neg_point(&self) -> Self;

    /// Multiplies the point at scalar value, assigns result to `self`
    fn scalar_mul_assign(&mut self, scalar: &Self::Scalar) {
        *self = self.scalar_mul(scalar)
    }
    /// Adds two points, assigns result to `self`
    fn add_point_assign(&mut self, other: &Self) {
        *self = self.add_point(other)
    }
    /// Substrates `other` from `self`, assigns result to `self`
    fn sub_point_assign(&mut self, other: &Self) {
        *self = self.sub_point(other)
    }
    /// Negates point, assigns result to `self`
    fn neg_point_assign(&mut self) {
        *self = self.neg_point()
    }

    /// Reference to underlying curve implementation
    fn underlying_ref(&self) -> &Self::Underlying;
    /// Mutual reference to underlying curve implementation
    fn underlying_mut(&mut self) -> &mut Self::Underlying;
    /// Construct a point from its underlying representation
    fn from_underlying(u: Self::Underlying) -> Self;
}

#[derive(Serialize, Deserialize)]
pub struct PointCoords {
    pub x: BigInt,
    pub y: BigInt,
}

#[derive(Debug)]
pub struct DeserializationError;

impl fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to deserialize the point")
    }
}

impl std::error::Error for DeserializationError {}

#[derive(Debug)]
pub struct NotOnCurve;

impl fmt::Display for NotOnCurve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "point not on the curve")
    }
}

impl std::error::Error for NotOnCurve {}
