use ultraviolet::Mat4;

#[derive(Clone, Copy, Debug)]
pub enum Projection {
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
    Perspective {
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    },
}

impl Into<Mat4> for Projection {
    fn into(self) -> Mat4 {
        match self {
            Projection::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => {
                ultraviolet::projection::orthographic_wgpu_dx(
                    left,
                    right,
                    bottom,
                    top,
                    near,
                    far,
                )
            }
            Projection::Perspective {
                fov,
                aspect_ratio,
                near,
                far,
            } => {
                ultraviolet::projection::perspective_wgpu_dx(
                    fov,
                    aspect_ratio,
                    near,
                    far,
                )
            }
        }
    }
}

impl Into<mint::ColumnMatrix4<f32>> for Projection {
    fn into(self) -> mint::ColumnMatrix4<f32> {
        let matrix: Mat4 = self.into();

        matrix.into()
    }
}
