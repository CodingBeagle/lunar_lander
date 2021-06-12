pub mod beagle_math
{
    pub struct Vector3
    {
        x: f32,
        y: f32,
        z: f32
    }

    pub struct Mat4
    {
        matrix: [f32; 16]
    }

    // Methods in Rust are defined within an "implementation block" for a specified struct.
    impl Mat4
    {
        // An "associated function" is like a static method, in that it is not associated with a specific instance of a struct,
        // but associated with the type itself.
        pub fn new(matrix: [f32; 16]) -> Mat4 {
            Mat4 {
                matrix
            }
        }

        pub fn identity() -> Mat4 {
            Mat4 {
                matrix: [
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0
                ]
            }
        }

        pub fn translate(pos: &Vector3) -> Mat4 {
            Mat4 {
                matrix: [
                    1.0  , 0.0  , 0.0  , pos.x,
                    0.0  , 1.0  , 0.0  , pos.y,
                    0.0  , 0.0  , 1.0  , pos.z,
                    0.0  , 0.0  , 1.0  , 1.0
                ]
            }
        }

        pub fn scale(scale: &Vector3) -> Mat4 {
            Mat4 {
                matrix: [
                    scale.x, 0.0    , 0.0    , 0.0,
                    0.0    , scale.y, 0.0    , 0.0,
                    0.0    , 0.0    , scale.z, 0.0,
                    0.0    , 0.0    , 0.0    , 1.0
                ]
            }
        }

        pub fn rotate_x(rad: f32) -> Mat4 {
            Mat4 {
                matrix: [
                    1.0, 0.0, 0.0, 0.0,
                    0.0, rad.cos(), -rad.sin(), 0.0,
                    0.0, rad.sin(), rad.cos(), 0.0,
                    0.0, 0.0, 0.0, 1.0
                ]
            }
        }

        pub fn rotate_y(rad: f32) -> Mat4 {
            Mat4 {
                matrix: [
                    rad.cos(), 0.0, rad.sin(), 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    -rad.sin(), 0.0, rad.cos(), 0.0,
                    0.0, 0.0, 0.0, 1.0
                ]
            }
        }

        pub fn rotate_z(rad: f32) -> Mat4 {
            Mat4 {
                matrix: [
                    rad.cos(), -rad.sin(), 0.0, 0.0,
                    rad.sin(), rad.cos(), 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0
                ]
            }
        }

        // For this projection matrix, I use what is sometimes referred to as the Hor+ scaling method for Field of View (https://en.wikipedia.org/wiki/Field_of_view_in_video_games).
        // Basically, the vertical FoV is fixed, while the horizontal FoV scales with the aspect ratio.
        pub fn projection(fov: f32, width: f32, height: f32, near: f32, far: f32) -> Mat4 {
            let y_scale = 1.0 / (fov * 0.5).tan();
            let x_scale = y_scale / (width / height);
            let q = far / (far - near);

            Mat4 {
                matrix: [
                    x_scale, 0.0, 0.0, 0.0,
                    0.0, y_scale, 0.0, 0.0,
                    0.0, 0.0, q, 1.0,
                    0.0, 0.0, -(q * near), 0.0
                ]
            }
        }

        pub fn get_value(&self) -> [f32; 16] {
            self.matrix
        }

        pub fn get_column_major_value(&self) -> [f32; 16] {
            [
                self.matrix[0], self.matrix[4], self.matrix[8], self.matrix[12],
                self.matrix[1], self.matrix[5], self.matrix[9], self.matrix[13],
                self.matrix[2], self.matrix[6], self.matrix[10], self.matrix[14],
                self.matrix[3], self.matrix[7], self.matrix[11], self.matrix[15]
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::beagle_math::*;

    #[test]
    fn should_return_correct_identity_matrix_when_constructing_identity_matrix() {
        // Arrange
        let expected_matrix: [f32; 16] = [1.0, 0.0, 0.0, 0.0,
                                          0.0, 1.0, 0.0, 0.0,
                                          0.0, 0.0, 1.0, 0.0,
                                          0.0, 0.0, 0.0, 1.0];

        // Act
        let value =  beagle_math::Mat4::identity().get_value();

        // Assert
        assert!( value.iter().eq(expected_matrix.iter()), "The identity matrix was found to be incorrect!");
    }

    #[test]
    fn should_return_correct_column_major_matrix_when_getting_column_major_value() {
        // Arrange
        let original_matrix = beagle_math::Mat4::new(
                                   [11.0, 12.0, 13.0, 14.0,
                                          21.0, 22.0, 23.0, 24.0,
                                          31.0, 32.0, 33.0, 34.0,
                                          41.0, 42.0, 43.0, 44.0]);

        let expected_matrix: [f32; 16] = [11.0, 21.0, 31.0, 41.0,
                                          12.0, 22.0, 32.0, 42.0,
                                          13.0, 23.0, 33.0, 43.0,
                                          14.0, 24.0, 34.0, 44.0];

        // Act
        let matrix_in_column_major = original_matrix.get_column_major_value();

        // Assert
        assert!( matrix_in_column_major.iter().eq(expected_matrix.iter()) );
    }
}