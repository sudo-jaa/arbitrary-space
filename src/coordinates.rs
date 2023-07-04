use num::PrimInt;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Sub;

use num::traits::Pow;
use serde::Serialize;

/// Calculate a unique hash for a given coordinate value
fn calculate_hash<T: Hash>(t: T) -> u64
where
    T: PrimInt,
{
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Generate a unique hash for a coordinate system
fn generate_hash(x: i64, y: i64, z: i64) -> u64 {
    let h_x = calculate_hash(x + 1);
    let h_y = calculate_hash(y + 2);
    let h_z = calculate_hash(z + 3);
    let mut res: i128 = h_x as i128;
    res = res + h_y as i128;
    res = res + h_z as i128;
    calculate_hash(res)
}

#[derive(Debug, Serialize, Copy, Clone)]
/// Represents a cartesian coordinate system in three dimensions
/// with a unique hash computed from each dimension value
pub struct Coordinates {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub hash: u64,
}

impl Coordinates {
    /// Create a new system of coordinates
    pub fn new(x: i64, y: i64, z: i64) -> Coordinates {
        let h = generate_hash(x, y, z);
        Coordinates { x, y, z, hash: h }
    }

    /// get the distance between two coordinates
    pub fn get_distance(coordinates_alpha: &Coordinates, coordinates_beta: &Coordinates) -> f32 {
        let alpha: f32 = (coordinates_alpha.x as f32 - coordinates_beta.x as f32).pow(2);
        let beta: f32 = (coordinates_alpha.y as f32 - coordinates_beta.y as f32).pow(2);
        let zeta: f32 = (coordinates_alpha.z as f32 - coordinates_beta.z as f32).pow(2);

        (alpha + beta + zeta).sqrt()
    }
}

impl PartialEq for Coordinates {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Sub for Coordinates {
    type Output = f64;

    fn sub(self, rhs: Self) -> Self::Output {
        f64::sqrt(
            f64::pow((self.x as f64) - (rhs.x as f64), 2)
                + f64::pow((self.y as f64) - (rhs.y as f64), 2)
                + f64::pow((self.z as f64) - (rhs.z as f64), 2),
        )
    }
}
