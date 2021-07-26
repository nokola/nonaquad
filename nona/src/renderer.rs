pub use crate::context::{CompositeOperationState, ImageId, Path, Vertex};
pub use crate::*;

#[derive(Debug, Copy, Clone)]
pub enum TextureType {
    RGBA,
    Alpha,
}

#[derive(Debug, Copy, Clone)]
pub struct Scissor {
    pub xform: Transform,
    pub extent: Extent,
}

pub trait Renderer {
    fn edge_antialias(&self) -> bool;

    fn view_size(&self) -> (f32, f32);
    
    fn device_pixel_ratio(&self) -> f32;

    fn create_texture(
        &mut self,
        texture_type: TextureType,
        width: usize,
        height: usize,
        flags: ImageFlags,
        data: Option<&[u8]>,
    ) -> Result<ImageId, NonaError>;

    fn delete_texture(&mut self, img: ImageId) -> Result<(), NonaError>;

    fn update_texture(
        &mut self,
        img: ImageId,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        data: &[u8],
    ) -> Result<(), NonaError>;

    fn texture_size(&self, img: ImageId) -> Result<(usize, usize), NonaError>;

    fn viewport(&mut self, extent: Extent, device_pixel_ratio: f32) -> Result<(), NonaError>;

    fn clear_screen(&mut self, color: Color);

    fn flush(&mut self) -> Result<(), NonaError>;

    fn fill(
        &mut self,
        paint: &Paint,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
        fringe: f32,
        bounds: Bounds,
        paths: &[Path],
    ) -> Result<(), NonaError>;

    fn stroke(
        &mut self,
        paint: &Paint,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[Path],
    ) -> Result<(), NonaError>;

    fn triangles(
        &mut self,
        paint: &Paint,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
        vertexes: &[Vertex],
    ) -> Result<(), NonaError>;
}
