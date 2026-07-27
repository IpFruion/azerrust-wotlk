use std::f32::consts::PI;

#[cxx::bridge]
mod ffi {
    #[namespace = "geometry"]
    extern "Rust" {
        #[cxx_name = "getAngle"]
        fn get_angle(start_x: f32, start_y: f32, dest_x: f32, dest_y: f32) -> f32;

        #[cxx_name = "getSlopeAngle"]
        fn get_slope_angle(
            start_x: f32,
            start_y: f32,
            start_z: f32,
            dest_x: f32,
            dest_y: f32,
            dest_z: f32,
        ) -> f32;

        #[cxx_name = "getSlopeAngleAbs"]
        fn get_slope_angle_abs(
            start_x: f32,
            start_y: f32,
            start_z: f32,
            dest_x: f32,
            dest_y: f32,
            dest_z: f32,
        ) -> f32;
    }
}

/// Get Angle between two vectors
pub fn get_angle(start_x: f32, start_y: f32, dest_x: f32, dest_y: f32) -> f32 {
    let dx = dest_x - start_x;
    let dy = dest_y - start_y;
    match dy.atan2(dx) {
        ang if ang >= 0.0 => ang,
        ang => 2.0 * PI + ang,
    }
}

/// Get Slope Angle
pub fn get_slope_angle(
    start_x: f32,
    start_y: f32,
    start_z: f32,
    dest_x: f32,
    dest_y: f32,
    dest_z: f32,
) -> f32 {
    let floor_dist = ((start_y - dest_y).powi(2) + (start_x - dest_x).powi(2)).sqrt();

    if floor_dist.abs() < 0.001 {
        return 0.0;
    }

    ((dest_z - start_z).abs() / floor_dist.abs()).atan()
}

pub fn get_slope_angle_abs(
    start_x: f32,
    start_y: f32,
    start_z: f32,
    dest_x: f32,
    dest_y: f32,
    dest_z: f32,
) -> f32 {
    get_slope_angle(start_x, start_y, start_z, dest_x, dest_y, dest_z)
}
