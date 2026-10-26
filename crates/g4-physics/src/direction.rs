//! Direction algebra copied from CLHEP (`source/externals/clhep`), including its quirks.

use g4_core::vec3::Vec3;

/// `Hep3Vector::orthogonal()` (`include/CLHEP/Vector/ThreeVector.icc:286-295`): a vector
/// orthogonal to `v` built from its two largest components. **It is not normalised**: for a unit
/// `v` its norm is `sqrt(1 - c^2)` where `c` is the smallest-magnitude component of `v`
/// (between `sqrt(2/3)` and 1). `G4DNAChampionElasticModel::SampleSecondaries` uses it as an
/// axis without normalising (see [`crate::models::ElasticRotation`]).
pub fn clhep_orthogonal(v: Vec3) -> Vec3 {
    let (xx, yy, zz) = (v.x.abs(), v.y.abs(), v.z.abs());
    if xx < yy {
        if xx < zz {
            Vec3::new(0.0, v.z, -v.y)
        } else {
            Vec3::new(v.y, -v.x, 0.0)
        }
    } else if yy < zz {
        Vec3::new(-v.z, 0.0, v.x)
    } else {
        Vec3::new(v.y, -v.x, 0.0)
    }
}

/// `Hep3Vector::rotateUz` (`src/ThreeVector.cc:33-53`): express `local`, given in a frame whose
/// z axis is `new_z` (which must be a unit vector), in the laboratory frame.
pub fn rotate_uz(local: Vec3, new_z: Vec3) -> Vec3 {
    let (u1, u2, u3) = (new_z.x, new_z.y, new_z.z);
    let up = u1 * u1 + u2 * u2;
    if up > 0.0 {
        let up = up.sqrt();
        Vec3::new(
            (u1 * u3 * local.x - u2 * local.y) / up + u1 * local.z,
            (u2 * u3 * local.x + u1 * local.y) / up + u2 * local.z,
            -up * local.x + u3 * local.z,
        )
    } else if u3 < 0.0 {
        Vec3::new(-local.x, local.y, -local.z)
    } else {
        local
    }
}

/// Scattering-angle cosine `cos(theta)` and azimuth `phi` to a direction in the frame of `axis`
/// (`sin(theta) cos(phi), sin(theta) sin(phi), cos(theta)` rotated with [`rotate_uz`]), as
/// `G4DNABornAngle::SampleDirectionForShell` (`:102-108`).
pub fn direction_from_angles(cos_theta: f64, phi: f64, axis: Vec3) -> Vec3 {
    let sin_theta = ((1.0 - cos_theta) * (1.0 + cos_theta)).sqrt();
    rotate_uz(
        Vec3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta),
        axis,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn orthogonal_is_orthogonal_and_not_always_unit() {
        for v in [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.6, 0.0, 0.8),
            Vec3::new(1.0, 1.0, 1.0).normalised().unwrap(),
            Vec3::new(-0.3, 0.5, -0.8124038404635961)
                .normalised()
                .unwrap(),
        ] {
            let o = clhep_orthogonal(v);
            assert!(v.dot(o).abs() < 1e-15, "{v:?} . {o:?}");
        }
        // along an axis the result is a unit vector
        assert_relative_eq!(clhep_orthogonal(Vec3::new(0.0, 0.0, 1.0)).norm(), 1.0);
        // body diagonal: smallest component 1/sqrt(3) -> norm sqrt(2/3)
        let d = Vec3::new(1.0, 1.0, 1.0).normalised().unwrap();
        assert_relative_eq!(
            clhep_orthogonal(d).norm(),
            (2.0f64 / 3.0).sqrt(),
            epsilon = 1e-15
        );
    }

    #[test]
    fn orthogonal_follows_the_clhep_branches() {
        // |x| < |y|, |x| < |z|  -> (0, z, -y)
        assert_eq!(
            clhep_orthogonal(Vec3::new(0.1, 0.5, 0.7)),
            Vec3::new(0.0, 0.7, -0.5)
        );
        // |x| < |y|, |x| >= |z|  -> (y, -x, 0)
        assert_eq!(
            clhep_orthogonal(Vec3::new(0.3, 0.5, 0.2)),
            Vec3::new(0.5, -0.3, 0.0)
        );
        // |x| >= |y|, |y| < |z|  -> (-z, 0, x)
        assert_eq!(
            clhep_orthogonal(Vec3::new(0.5, 0.1, 0.7)),
            Vec3::new(-0.7, 0.0, 0.5)
        );
        // |x| >= |y|, |y| >= |z| -> (y, -x, 0)
        assert_eq!(
            clhep_orthogonal(Vec3::new(0.7, 0.5, 0.1)),
            Vec3::new(0.5, -0.7, 0.0)
        );
    }

    #[test]
    fn rotate_uz_preserves_norm_and_maps_z_to_axis() {
        let axis = Vec3::new(0.3, -0.5, 0.8).normalised().unwrap();
        let z = rotate_uz(Vec3::new(0.0, 0.0, 1.0), axis);
        assert_relative_eq!(z.x, axis.x, epsilon = 1e-15);
        assert_relative_eq!(z.y, axis.y, epsilon = 1e-15);
        assert_relative_eq!(z.z, axis.z, epsilon = 1e-15);
        let v = rotate_uz(Vec3::new(0.2, -0.7, 0.4), axis);
        assert_relative_eq!(v.norm(), Vec3::new(0.2, -0.7, 0.4).norm(), epsilon = 1e-15);
    }

    #[test]
    fn rotate_uz_handles_the_polar_axes() {
        let v = Vec3::new(0.2, -0.7, 0.4);
        assert_eq!(rotate_uz(v, Vec3::new(0.0, 0.0, 1.0)), v);
        assert_eq!(
            rotate_uz(v, Vec3::new(0.0, 0.0, -1.0)),
            Vec3::new(-0.2, -0.7, -0.4)
        );
    }

    #[test]
    fn angle_to_axis_is_the_requested_polar_angle() {
        let axis = Vec3::new(1.0, 2.0, -0.5).normalised().unwrap();
        for cos_theta in [-1.0, -0.4, 0.0, 0.3, 0.999] {
            for phi in [0.0, 1.0, 4.0] {
                let d = direction_from_angles(cos_theta, phi, axis);
                assert_relative_eq!(d.dot(axis), cos_theta, epsilon = 1e-14);
                assert_relative_eq!(d.norm(), 1.0, epsilon = 1e-14);
            }
        }
    }
}
