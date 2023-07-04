mod coordinates;

use crate::coordinates::Coordinates;
use std::f64::consts::PI;
use uom::si::angle::radian;
use uom::si::f64::Angle;
use uom::si::f64::Length;
use uom::si::length::{light_year, meter};

static TO_RADIAN_MODIFIER: f64 = 180.0 / PI;

/// A collection of functions that compute relations between visual angle, distance, and size of an object.
struct VisualAngle;
impl VisualAngle {
    /// Gets the visual angle of an object from the distance and size of the object
    fn angle_from_distance_size(distance: &Length, size: &Length) -> Angle {
        let angle = 2.0 * f64::atan((size.value / 2.0) / distance.value);
        Angle::new::<radian>(angle)
    }

    /// Gets the distance of an object from its visual angle and size
    fn distance_from_visual_angle_and_size(visual_angle: &Angle, size: &Length) -> Length {
        Length::new::<meter>((size.value / 2.0) / (f64::tan(visual_angle.value / 2.0)))
    }

    /// Gets the size of an object from its visual angle and distance
    fn size_from_visual_angle_and_distance(angle: &Angle, distance: &Length) -> Length {
        Length::new::<meter>(2.0 * distance.value * f64::tan((angle.value / 2.0)))
    }
}

#[derive(Debug, Clone)]
enum Shape {
    // A spherical object! All large gravitationally bound bodies tend to go spherical anyway
    // but we'll maybe add more later
    Sphere(Length),
}

impl Shape {
    /// return the visual angle of a shape from a specified distance
    fn get_visual_angle(&self, distance: Length) -> Angle {
        match self {
            Shape::Sphere(size) => VisualAngle::angle_from_distance_size(&distance, size),
        }
    }
}

#[derive(Debug)]
/// An object represented in cartesian space
pub struct Object {
    position: Coordinates,
    shape: Shape,
}

impl Object {
    fn new(position: Coordinates, shape: Shape) -> Self {
        Object { position, shape }
    }
}

/// Representation of an object observed inside the layout
pub struct ObservedObject {
    /// The shape of the observed object
    shape: Shape,
    /// The visual angle (size) of the object at it's position
    visual_angle: Angle,
    /// The actual coordinates in the layout of the object
    coordinates: Coordinates,
    /// The position from which this object was observed
    observed_from: Coordinates,
}

#[derive(Debug)]
/// A three-dimensional cartesian space containing a number of `Object`s that can simulate
/// an arbitrarily large space by reorganising objects within it according to their apparent visual angle.
///
/// In practice, this means that a layout is a 3d coordinate space that is encoded to represent
/// a size of any magnitude, where each object contained within it can have it's visual angle
/// computed as though it existed properly within the space of the encoded size.
///
/// It's a bit like spoofing the rendering of a very large static scene in 3d without having
/// to deal with those pesky floating point errors.
///
/// A layout is initialised with the following properties:
/// coordinate_bound: the number of unit-less steps that the layout extends in any given dimension. Higher values will provide a higher resolution layout.
/// dimension: the actual edge length that this layout represents in real terms. Often this is analagous to the actual size of the space being described
/// objects: the representation of objects contained within the layout
pub struct Layout {
    objects: Vec<Object>,
    coordinate_bound: i64,
    dimension: Length,
}

impl Layout {
    pub fn new(coordinate_bound: i64, dimension: Length) -> Self {
        Layout {
            coordinate_bound,
            dimension,
            ..Default::default()
        }
    }

    /// Checks to see if a dimension is valid or if it exceeds the boundaries of the layout.
    fn check_bound(&self, dimension_value: &i64) -> bool {
        let start = self.coordinate_bound.overflowing_neg().0;
        let end = self.coordinate_bound;
        &start <= dimension_value && dimension_value <= &end
    }

    /// Adds an object to the layout
    pub fn add_object(&mut self, object: Object) -> bool {
        if !self.check_bound(&object.position.x)
            || !self.check_bound(&object.position.y)
            || !self.check_bound(&object.position.z)
        {
            false
        } else {
            self.objects.push(object);
            true
        }
    }

    /// Gets the distance between two coordinates in the layout
    fn get_distance(&self, position: &Coordinates, comparison: &Coordinates) -> Length {
        let length_of_unit = self.dimension / (self.coordinate_bound * 2) as f64;
        let distance = Coordinates::get_distance(position, comparison);

        length_of_unit * distance as f64
    }

    /// Produce a vector of all objects within the layout and their observed sizes relative to an
    /// observer position
    pub fn observe_layout_objects(&self, origin: &Coordinates) -> Vec<ObservedObject> {
        self.objects
            .iter()
            .map(|object| {
                let distance = self.get_distance(origin, &object.position);
                ObservedObject {
                    shape: object.shape.clone(),
                    visual_angle: object.shape.get_visual_angle(distance),
                    coordinates: object.position,
                    observed_from: *origin,
                }
            })
            .collect()
    }
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            objects: vec![],
            coordinate_bound: 1000,
            dimension: Length::new::<light_year>(1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use uom::si::{angle::degree, length::kilometer};

    use super::*;

    fn approx_equal(a: f64, b: f64, decimal_places: u8) -> bool {
        let factor = 10.0f64.powi(decimal_places as i32);
        let a = (a * factor).trunc();
        let b = (b * factor).trunc();
        a == b
    }

    #[test]
    fn visual_angle_from_distance_and_size() {
        let distance = Length::new::<meter>(1.0);
        let size = Length::new::<meter>(1.0);
        let angle = VisualAngle::angle_from_distance_size(&distance, &size);

        assert!(approx_equal(
            angle.value,
            Angle::new::<degree>(53.1301).value,
            6
        ));
    }

    #[test]
    fn size_from_visual_angle_and_distance() {
        let distance = Length::new::<meter>(1.0);
        let angle = Angle::new::<degree>(10.0);
        let size = VisualAngle::size_from_visual_angle_and_distance(&angle, &distance);

        assert!(approx_equal(
            size.value,
            Length::new::<meter>(0.174977).value,
            6
        ));
    }

    #[test]
    fn distance_from_visual_angle_and_size() {
        let size = Length::new::<meter>(1.0);
        let angle = Angle::new::<degree>(10.0);
        let distance = VisualAngle::distance_from_visual_angle_and_size(&angle, &size);

        assert!(approx_equal(
            distance.value,
            Length::new::<meter>(5.715026).value,
            6
        ));
    }

    #[test]
    fn visual_angle() {
        let shape = Shape::Sphere(Length::new::<meter>(1.0));
        let distance = Length::new::<meter>(1.0);

        let angle = shape.get_visual_angle(distance);
        assert!(
            approx_equal(angle.value, Angle::new::<degree>(53.1301).value, 4),
            "actual: {}, test: {}",
            angle.value,
            Angle::new::<degree>(11.421186).value
        );
    }

    #[test]
    fn placement() {
        let mut layout = Layout::new(5, Length::new::<kilometer>(1000.0));
        let object_1 = Object::new(
            Coordinates::new(0, 0, 0),
            Shape::Sphere(Length::new::<kilometer>(1000.0)),
        );
        let success = layout.add_object(object_1);

        assert!(success, "failed adding 000");

        let object_2 = Object::new(
            Coordinates::new(5, 0, 0),
            Shape::Sphere(Length::new::<kilometer>(1000.0)),
        );
        let success = layout.add_object(object_2);

        assert!(success, "failed adding 500");

        let object_3 = Object::new(
            Coordinates::new(-4, 0, 0),
            Shape::Sphere(Length::new::<kilometer>(1000.0)),
        );
        let success = layout.add_object(object_3);

        assert!(success, "failed adding -400");

        let object_4 = Object::new(
            Coordinates::new(6, 0, 0),
            Shape::Sphere(Length::new::<kilometer>(1000.0)),
        );
        let success = layout.add_object(object_4);

        assert_eq!(success, false, "failed adding 600");

        let object_5 = Object::new(
            Coordinates::new(-6, 0, 0),
            Shape::Sphere(Length::new::<kilometer>(1000.0)),
        );
        let success = layout.add_object(object_5);

        assert_eq!(success, false, "failed adding -600");
    }

    #[test]
    fn basic_object() {
        let mut layout = Layout::default();
        let object_1 = Object::new(
            Coordinates::new(5, 0, 0),
            Shape::Sphere(Length::new::<kilometer>(1000.0)),
        );
        layout.add_object(object_1);

        let distance = layout.get_distance(&Coordinates::new(0, 0, 0), &Coordinates::new(5, 0, 0));
    }

    #[test]
    fn moon_test() {
        let mut layout = Layout::new(5, Length::new::<kilometer>(384400.0 * 10.0));
        let moon = Object::new(
            Coordinates::new(1, 0, 0),
            Shape::Sphere(Length::new::<kilometer>(3474.8)),
        );
        layout.add_object(moon);

        let distance = layout.get_distance(&Coordinates::new(0, 0, 0), &Coordinates::new(1, 0, 0));

        let angle = layout
            .objects
            .pop()
            .unwrap()
            .shape
            .get_visual_angle(distance)
            .value;
        assert!(
            approx_equal(angle, Angle::new::<degree>(0.517924).value, 6),
            "{}",
            angle
        );
    }

    #[test]
    fn system_test() {
        let mut layout = Layout::new(2992000000, Length::new::<kilometer>(5.984e9));

        let moon = Object::new(
            Coordinates::new(384399, 0, 0),
            Shape::Sphere(Length::new::<kilometer>(3474.8)),
        );
        layout.add_object(moon);

        let sun = Object::new(
            Coordinates::new(149597870, 0, 0),
            Shape::Sphere(Length::new::<kilometer>(1392700.0)),
        );
        layout.add_object(sun);

        layout.objects.iter().for_each(|object| {
            let distance = layout.get_distance(&Coordinates::new(0, 0, 0), &object.position);
        });

        // let distance = layout.get_distance(&Coordinates::new(0, 0, 0), &Coordinates::new(1, 0, 0));
        // assert!(approx_equal(layout.objects.get(0).unwrap().shape.get_visual_angle(distance).value, 0.517924, 6));
    }
}
