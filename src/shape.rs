use std::{f64, sync::Arc};

use glam::{DMat3, DMat4, DVec3, DVec4, Vec4Swizzles};
use rand::rngs::StdRng;

use crate::{
    aabb::Aabb,
    interval::Interval,
    material::Material,
    math::{Axis, DPoint3, Ray},
};

pub mod cube;
pub mod quad;
pub mod sphere;

pub trait Hittable: Send + Sync {
    /// Used for `HitRecord` of incident ray.
    fn intersect(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord>;

    /// Return a random point, normal and the pdf.
    /// The function is a combination of `pdf` and `random` in Ray Tracing Series 3.
    fn sample(
        &self,
        _target: DPoint3,
        _rng: &mut StdRng,
        _shutter_time: f64,
    ) -> (DPoint3, DVec3, f64) {
        (
            DPoint3::splat(1.0),
            DVec3::splat(1.0),
            1.0 / (2.0 * f64::consts::PI),
        )
    }
}

pub trait Bounded: Hittable {
    /// The bounding box of the shape.
    fn bbox(&self) -> Aabb;
}

#[derive(Default, Clone)]
pub struct HitRecord {
    /// The 3d coordinations of intersection point.
    pub p: DPoint3,

    /// Time which can be used to compute point along the ray through the formula
    /// p = origin + t * direction. This attribute is more microscopic than the time of
    /// the `Ray` structure.
    pub t: f64,

    /// The 3d coordinations of the normal vector in the intersection surface towards
    /// the incident ray.
    pub normal: DVec3,

    /// The flag to determine whether the normal vector towards you. e.g. if the radius is
    /// negative, then the normal vector is inverted.
    pub front_face: bool,

    /// The material of intersect object.
    pub material: Option<Arc<Material>>,

    /// The coordinates of the object surface mapping to the texture map
    pub u: f64,
    pub v: f64,
}

impl HitRecord {
    /// Set the normal vector of intersections surface which face to the incident ray.
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: DVec3) {
        self.front_face = r.dir.dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }

    pub fn material(&self) -> &Material {
        self.material.as_ref().unwrap()
    }
}

/// A Object that has been composed with a transformation.
pub struct Transformed<T> {
    /// The hittable shape that need to transforme.
    shape: T,

    /// The transformation matrix to transform object.
    transform: DMat4,

    /// The inverse of `transform` which use to transform the incident ray.
    inverse_transform: DMat4,

    #[allow(dead_code)]
    /// The transformation which extract from `transform` and not contains translate part.
    linear: DMat3,

    /// The inverse and transpose of `transform` which use to rectify normal vector.
    normal_transform: DMat3,
}

impl<T> Transformed<T> {
    pub fn new(shape: T, transform: DMat4) -> Self {
        let linear = DMat3::from_mat4(transform);
        let inverse_transform = transform.inverse();
        let normal_transform = linear.inverse().transpose();
        Self {
            shape,
            transform,
            linear,
            inverse_transform,
            normal_transform,
        }
    }
}

impl<T: Hittable> Hittable for Transformed<T> {
    fn intersect(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let ray_trans = r.apply_transform(&self.inverse_transform);
        match self.shape.intersect(&ray_trans, ray_t) {
            None => None,
            Some(mut rec) => {
                // Transform intersection point back to world space
                let p_world = self.transform * rec.p.extend(1.0);
                rec.p = p_world.xyz();

                // Fix normal vector by multiplying by M^-T
                rec.normal = self.normal_transform.mul_vec3(rec.normal).normalize();

                Some(rec)
            }
        }
    }
}

impl<T: Bounded> Bounded for Transformed<T> {
    fn bbox(&self) -> Aabb {
        let Aabb { x, y, z } = self.shape.bbox();

        // Transform all 8 corners and find aabb of transformed shape.
        let corners = [
            (x.min, y.min, z.min),
            (x.min, y.min, z.max),
            (x.min, y.max, z.min),
            (x.min, y.max, z.max),
            (x.max, y.min, z.min),
            (x.max, y.min, z.max),
            (x.max, y.max, z.min),
            (x.max, y.max, z.max),
        ];

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut min_z = f64::INFINITY;
        let mut max_z = f64::NEG_INFINITY;

        for (cx, cy, cz) in corners {
            let transformed = self.transform.mul_vec4(DVec4::new(cx, cy, cz, 1.0)).xyz();
            min_x = min_x.min(transformed.x);
            max_x = max_x.max(transformed.x);
            min_y = min_y.min(transformed.y);
            max_y = max_y.max(transformed.y);
            min_z = min_z.min(transformed.z);
            max_z = max_z.max(transformed.z);
        }

        Aabb::new(
            Interval::new(min_x, max_x),
            Interval::new(min_y, max_y),
            Interval::new(min_z, max_z),
        )
    }
}

pub trait Transformable<T> {
    /// Translate the shape from vector.
    fn translate(self, v: DVec3) -> Transformed<T>;

    /// Rotate the shape from a specified angle in radians.
    fn rotate(self, axis: Axis, angle: f64) -> Transformed<T>;
}

impl<T: Hittable> Transformable<T> for T {
    fn translate(self, v: DVec3) -> Transformed<T> {
        Transformed::new(self, DMat4::from_translation(v.into()))
    }
    fn rotate(self, axis: Axis, angle: f64) -> Transformed<T> {
        let axis_vec = match axis {
            Axis::X => DVec3::new(1.0, 0.0, 0.0),
            Axis::Y => DVec3::new(0.0, 1.0, 0.0),
            Axis::Z => DVec3::new(0.0, 0.0, 1.0),
        };
        Transformed::new(self, DMat4::from_axis_angle(axis_vec, angle))
    }
}
