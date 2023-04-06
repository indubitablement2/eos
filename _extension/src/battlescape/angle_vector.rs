use super::*;

pub trait VectorAngle {
    /// Angle relative to the (0, 1) vector.
    fn angle_x(self) -> f32;
    /// The angle such that self rotated by this angle point in the same direction as to.
    /// Result is in range `]-pi, pi[`.
    fn angle_to(self, to: Self) -> f32;
    /// The rotation such that `self` * rotation point in the same direction as to.
    fn rotation_to(self, to: Self) -> na::UnitComplex<f32>;
}
impl VectorAngle for na::Vector2<f32> {
    fn angle_x(self) -> f32 {
        na::RealField::atan2(self.y, self.x)
    }

    fn angle_to(self, to: Self) -> f32 {
        // to.angle_x() - self.angle_x()
        // Avoid one call to atan2 and angle is within ]-pi, pi[.
        // atan2(cross, dot)
        na::RealField::atan2(self.x * to.y - self.y * to.x, self.x * to.x + self.y * to.y)
    }

    fn rotation_to(self, to: Self) -> na::UnitComplex<f32> {
        Rotation::new(self.angle_to(to))
    }
}

#[test]
fn test_vector_angle() {
    fn rv() -> na::Vector2<f32> {
        na::vector![
            random::<f32>().mul_add(2.0, -1.0),
            random::<f32>().mul_add(2.0, -1.0)
        ] * 10.0
    }
    for _ in 0..1000 {
        let from = rv();
        let to = rv();
        let from_scaled = from.normalize() * to.magnitude();
        let result = from.rotation_to(to).transform_vector(&from_scaled);
        let dif = to - result;
        assert!(
            dif.x.abs() < 0.01 && dif.y < 0.01,
            "\nfrom:{}to:{}result:{}dif:{}\na:{}",
            from,
            to,
            result,
            dif,
            from.angle_to(to),
        );
    }
}
