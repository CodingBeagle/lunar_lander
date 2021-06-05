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