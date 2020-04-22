use nvg::renderer::*;
use slab::Slab;
use std::ffi::c_void;
use miniquad::graphics::*;
use miniquad::graphics::{ Context as MiniContext };
use glam::{Vec4, Mat4};

enum ShaderType {
    FillGradient,
    FillImage,
    Simple,
    Image,
}

#[derive(PartialEq, Eq)]
enum CallType {
    Fill,
    ConvexFill,
    Stroke,
    Triangles,
}

struct Blend(BlendState);

impl From<CompositeOperationState> for Blend {
    fn from(state: CompositeOperationState) -> Self {
        Blend(BlendState {
            eq_rgb: Equation::Add,
            eq_alpha: Equation::Add,
            src_rgb: convert_blend_factor(state.src_rgb),
            dst_rgb: convert_blend_factor(state.dst_rgb),
            src_alpha: convert_blend_factor(state.src_alpha),
            dst_alpha: convert_blend_factor(state.dst_alpha),
        })
    }
}

struct Call {
    call_type: CallType,
    image: Option<usize>,
    path_offset: usize,
    path_count: usize,
    triangle_offset: usize,
    triangle_count: usize,
    uniform_offset: usize,
    blend_func: Blend,
}

struct Texture {
    tex: miniquad::Texture,
    flags: ImageFlags,
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.tex.delete();
    }
}

struct GLPath {
    fill_offset: usize,
    fill_count: usize,
    stroke_offset: usize,
    stroke_count: usize,
}

pub struct Renderer<'a> {
    // shader: Shader,
    textures: Slab<Texture>, // TODO_REPLACE: bindings.images
    view: Extent,
    // vert_buf: GLuint, TODO_REMOVE
    // vert_arr: GLuint, TODO_REMOVE
    pipeline: Pipeline,
    bindings: Bindings,
    calls: Vec<Call>,
    paths: Vec<GLPath>,
    vertexes: Vec<Vertex>,
    indices: Vec<u16>,
    uniforms: Vec<shader::FragUniforms>,
    ctx: &'a mut MiniContext,
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = include_str!("shader.vert");
    pub const FRAGMENT: &str = include_str!("shader.frag");

    pub const ATTRIBUTES: &[VertexAttribute] = &[
        VertexAttribute::new("vertex", VertexFormat::Float2),
        VertexAttribute::new("tcoord", VertexFormat::Float2),
    ];
    pub const META: ShaderMeta = ShaderMeta {
        images: &["tex"],
        uniforms: UniformBlockLayout {
            uniforms: &[
                UniformDesc::new("viewSize", UniformType::Float2),

                UniformDesc::new("scissorMat", UniformType::Mat4),
                UniformDesc::new("paintMat", UniformType::Mat4),
                UniformDesc::new("innerCol", UniformType::Float4),
                UniformDesc::new("outerCol", UniformType::Float4),
                UniformDesc::new("scissorExt", UniformType::Float2),
                UniformDesc::new("scissorScale", UniformType::Float2),
                UniformDesc::new("extent", UniformType::Float2),
                UniformDesc::new("radius", UniformType::Float1),
                UniformDesc::new("feather", UniformType::Float1),
                UniformDesc::new("strokeMult", UniformType::Float1),
                UniformDesc::new("strokeThr", UniformType::Float1),
                UniformDesc::new("texType", UniformType::Int1),
                UniformDesc::new("type", UniformType::Int1),
            ],
        },
    };

    #[repr(C)]
    pub struct Uniforms {
        pub view_size: (f32, f32),
        pub frag: FragUniforms,
    }

    #[derive(Default)]
    #[repr(C)]
    pub struct FragUniforms {
        pub scissor_mat: glam::Mat4,
        pub paint_mat: glam::Mat4,
        pub inner_col: (f32, f32, f32, f32),
        pub outer_col: (f32, f32, f32, f32),
        pub scissor_ext: (f32, f32),
        pub scissor_scale: (f32, f32),
        pub extent: (f32, f32),
        pub radius: f32,
        pub feather: f32,
        pub stroke_mult: f32,
        pub stroke_thr: f32,
        pub tex_type: i32,
        pub type_: i32,
    }
}

const MAX_VERTICES: usize = 21845; // u16.max / 3 due to index buffer limitations
const MAX_INDICES: usize = u16::max_value() as usize;

impl<'a> Renderer<'a> {
    pub fn create(ctx: &mut MiniContext) -> anyhow::Result<Renderer> {
        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::META);
        let pipeline = Pipeline::with_params(ctx, &[BufferLayout::default()], shader::ATTRIBUTES, shader,
            PipelineParams {
                depth_write: false,
                color_blend: None, // set during draws
                color_write: (true, true, true, true),
                front_face_order: FrontFaceOrder::CounterClockwise,
                ..Default::default()
            },
        );

        let vertex_buffer = Buffer::stream(ctx, BufferType::VertexBuffer, MAX_VERTICES * std::mem::size_of::<Vertex>());
        let index_buffer = Buffer::stream(ctx, BufferType::IndexBuffer, MAX_INDICES * std::mem::size_of::<u16>());

        let pixels: [u8; 4 * 4 * 4] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
            0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let temp_texture = miniquad::Texture::from_rgba8(ctx, 4, 4, &pixels);
        
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![temp_texture], // TODO: set and use image only if needed
        };

        Ok(Renderer {
            ctx,
            pipeline,
            bindings,
            textures: Default::default(),
            view: Default::default(),
            calls: Default::default(),
            paths: Default::default(),
            vertexes: Default::default(),
            indices: Default::default(),
            uniforms: Default::default(),
        })
    }

    fn set_uniforms(ctx: &mut MiniContext, uniforms: &shader::FragUniforms, img: Option<usize>) {
        ctx.apply_uniforms(uniforms);

        // TODOKOLA: ADD support, see //     // TODO: set image in a better way!!! in flush()
        // if let Some(img) = img {
        //     if let Some(texture) = self.textures.get(img) {
        //         glBindTexture(GL_TEXTURE_2D, texture.tex);
        //     }
        // } else {
        //     glBindTexture(GL_TEXTURE_2D, 0);
        // }
    }

    unsafe fn do_fill(&self, call: &Call) {
        // TODOKOLA: ADD support
        // let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];

        // glEnable(GL_STENCIL_TEST);
        // glStencilMask(0xff);
        // glStencilFunc(GL_ALWAYS, 0, 0xff);
        // glColorMask(GL_FALSE, GL_FALSE, GL_FALSE, GL_FALSE);

        // self.set_uniforms(call.uniform_offset, call.image);

        // // glStencilOpSeparate(GL_FRONT, GL_KEEP, GL_KEEP, GL_INCR_WRAP);
        // // glStencilOpSeparate(GL_BACK, GL_KEEP, GL_KEEP, GL_DECR_WRAP);
        // glDisable(GL_CULL_FACE);
        // for path in paths {
        //     glDrawArrays(
        //         GL_TRIANGLE_FAN,
        //         path.fill_offset as i32,
        //         path.fill_count as i32,
        //     );
        // }
        // glEnable(GL_CULL_FACE);

        // glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);

        // self.set_uniforms(call.uniform_offset + 1, call.image);

        // glStencilFunc(GL_EQUAL, 0x00, 0xff);
        // glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);
        // for path in paths {
        //     glDrawArrays(
        //         GL_TRIANGLE_STRIP,
        //         path.stroke_offset as i32,
        //         path.stroke_count as i32,
        //     );
        // }

        // glStencilFunc(GL_NOTEQUAL, 0x00, 0xff);
        // glStencilOp(GL_ZERO, GL_ZERO, GL_ZERO);
        // glDrawArrays(
        //     GL_TRIANGLE_STRIP,
        //     call.triangle_offset as i32,
        //     call.triangle_count as i32,
        // );

        // glDisable(GL_STENCIL_TEST);
    }

    unsafe fn do_convex_fill(&self, call: &Call) {
        // TODOKOLA: ADD support
        // let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
        // self.set_uniforms(call.uniform_offset, call.image);
        // for path in paths {
        //     glDrawArrays(
        //         GL_TRIANGLE_FAN,
        //         path.fill_offset as i32,
        //         path.fill_count as i32,
        //     );
        //     if path.stroke_count > 0 {
        //         glDrawArrays(
        //             GL_TRIANGLE_STRIP,
        //             path.stroke_offset as i32,
        //             path.stroke_count as i32,
        //         );
        //     }
        // }
    }

    unsafe fn do_stroke(&self, call: &Call) {
        // TODOKOLA: ADD support
        // let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];

        // glEnable(GL_STENCIL_TEST);
        // glStencilMask(0xff);
        // glStencilFunc(GL_EQUAL, 0x0, 0xff);
        // glStencilOp(GL_KEEP, GL_KEEP, GL_INCR);
        // self.set_uniforms(call.uniform_offset + 1, call.image);
        // for path in paths {
        //     glDrawArrays(
        //         GL_TRIANGLE_STRIP,
        //         path.stroke_offset as i32,
        //         path.stroke_count as i32,
        //     );
        // }

        // self.set_uniforms(call.uniform_offset, call.image);
        // glStencilFunc(GL_EQUAL, 0x0, 0xff);
        // glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);
        // for path in paths {
        //     glDrawArrays(
        //         GL_TRIANGLE_STRIP,
        //         path.stroke_offset as i32,
        //         path.stroke_count as i32,
        //     );
        // }

        // glColorMask(GL_FALSE, GL_FALSE, GL_FALSE, GL_FALSE);
        // glStencilFunc(GL_ALWAYS, 0x0, 0xff);
        // glStencilOp(GL_ZERO, GL_ZERO, GL_ZERO);
        // for path in paths {
        //     glDrawArrays(
        //         GL_TRIANGLE_STRIP,
        //         path.stroke_offset as i32,
        //         path.stroke_count as i32,
        //     );
        // }
        // glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);

        // glDisable(GL_STENCIL_TEST);
    }

    unsafe fn do_triangles(&mut self, call: &Call) {
        Self::set_uniforms(self.ctx, &self.uniforms[call.uniform_offset], call.image);

        // TODO: update self.indices and self.vertexes
        self.bindings.index_buffer.update(self.ctx, &self.indices[..])

        // glDrawArrays(
        //     GL_TRIANGLES,
        //     call.triangle_offset as i32,
        //     call.triangle_count as i32,
        // );
    }

    fn convert_paint(
        &self,
        paint: &Paint,
        scissor: &Scissor,
        width: f32,
        fringe: f32,
        stroke_thr: f32,
    ) -> shader::FragUniforms {
        let mut frag = shader::FragUniforms {
            scissor_mat: Default::default(),
            paint_mat: Default::default(),
            inner_col: premul_color(paint.inner_color).into_tuple(),
            outer_col: premul_color(paint.outer_color).into_tuple(),
            scissor_ext: Default::default(),
            scissor_scale: Default::default(),
            extent: Default::default(),
            radius: 0.0,
            feather: 0.0,
            stroke_mult: 0.0,
            stroke_thr,
            tex_type: 0,
            type_: 0,
        };

        if scissor.extent.width < -0.5 || scissor.extent.height < -0.5 {
            frag.scissor_ext = (1.0, 1.0);
            frag.scissor_scale = (1.0, 1.0);
        } else {
            frag.scissor_mat = xform_to_4x4(scissor.xform.inverse());
            frag.scissor_ext = (scissor.extent.width, scissor.extent.height);
            frag.scissor_scale = ((scissor.xform.0[0] * scissor.xform.0[0]
                + scissor.xform.0[2] * scissor.xform.0[2]).sqrt() / fringe,
                (scissor.xform.0[1] * scissor.xform.0[1]
                + scissor.xform.0[3] * scissor.xform.0[3]).sqrt() / fringe);
        }

        frag.extent = (paint.extent.width, paint.extent.height);
        frag.stroke_mult = (width * 0.5 + fringe * 0.5) / fringe;

        let mut invxform = Transform::default();

        if let Some(img) = paint.image {
            if let Some(texture) = self.textures.get(img) {
                if texture.flags.contains(ImageFlags::FLIPY) {
                    let m1 = Transform::translate(0.0, frag.extent.1 * 0.5) * paint.xform;
                    let m2 = Transform::scale(1.0, -1.0) * m1;
                    let m1 = Transform::translate(0.0, -frag.extent.1 * 0.5) * m2;
                    invxform = m1.inverse();
                } else {
                    invxform = paint.xform.inverse();
                };

                frag.type_ = ShaderType::FillImage as i32;
                match texture.tex.format {
                    TextureFormat::RGBA8 => {
                        frag.tex_type = if texture.flags.contains(ImageFlags::PREMULTIPLIED) {
                            0
                        } else {
                            1
                        }
                    }
                    _ => { todo!("Unsupported texture type")},
                }
            }
        } else {
            frag.type_ = ShaderType::FillGradient as i32;
            frag.radius = paint.radius;
            frag.feather = paint.feather;
            invxform = paint.xform.inverse();
        }

        frag.paint_mat = xform_to_4x4(invxform);

        frag
    }

    fn append_uniforms(&mut self, uniforms: shader::FragUniforms) {
        self.uniforms.push(uniforms);
    }
}

trait IntoTuple4<T>{
    fn into_tuple(self) -> (T, T, T, T);
}

impl IntoTuple4<f32> for Color {
    fn into_tuple(self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }
}

impl renderer::Renderer for Renderer<'_> {
    fn edge_antialias(&self) -> bool {
        true
    }

    fn create_texture(
        &mut self,
        texture_type: TextureType,
        width: usize,
        height: usize,
        flags: ImageFlags,
        data: Option<&[u8]>,
    ) -> anyhow::Result<ImageId> {
        let tex: miniquad::Texture = miniquad::Texture::new(self.ctx, 
            TextureAccess::Static,
            data,
            TextureParams {
                format: match texture_type {
                    TextureType::RGBA => TextureFormat::RGBA8,
                    TextureType::Alpha => TextureFormat::RGBA8, // TODO: support alpha textures
                },
                wrap: TextureWrap::Clamp, // TODO: support repeatx/y/mirror
                filter: if flags.contains(ImageFlags::NEAREST) {
                    FilterMode::Nearest
                } else {
                    FilterMode::Linear
                },
                width: width as u32,
                height: height as u32,
            });

        // TODO: support ImageFlags::GENERATE_MIPMAPS) with/without if flags.contains(ImageFlags::NEAREST) {

        let id = self.textures.insert(Texture {
            tex,
            flags,
        });
        Ok(id)
    }

    fn delete_texture(&mut self, img: ImageId) -> anyhow::Result<()> {
        if let Some(texture) = self.textures.get(img) {
            texture.tex.delete();
            self.textures.remove(img);
            Ok(())
        } else {
            bail!("texture '{}' not found", img);
        }
    }

    fn update_texture(
        &mut self,
        img: ImageId,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        data: &[u8],
    ) -> anyhow::Result<()> {
        if let Some(texture) = self.textures.get(img) {
            texture.tex.update(self.ctx, data);
            Ok(())
        } else {
            bail!("texture '{}' not found", img);
        }
    }

    fn texture_size(&self, img: ImageId) -> anyhow::Result<(usize, usize)> {
        if let Some(texture) = self.textures.get(img) {
            Ok((texture.tex.width as usize, texture.tex.height as usize))
        } else {
            bail!("texture '{}' not found", img);
        }
    }

    fn viewport(&mut self, extent: Extent, _device_pixel_ratio: f32) -> anyhow::Result<()> {
        self.view = extent;
        Ok(())
    }

    fn cancel(&mut self) -> anyhow::Result<()> {
        self.vertexes.clear();
        self.paths.clear();
        self.calls.clear();
        self.uniforms.clear();
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        if self.calls.is_empty() {
            self.vertexes.clear();
            self.paths.clear();
            self.calls.clear();
            self.uniforms.clear();
           
            return Ok(());
        }
            unsafe {
                self.ctx.begin_default_pass(PassAction::clear_color(0.5, 0.5, 1.0, 1.0));

                // glUseProgram(self.shader.prog); DONE
                self.ctx.apply_pipeline(&self.pipeline);
                self.ctx.apply_bindings(&self.bindings);


                // glEnable(GL_CULL_FACE); // TODO: support in miniquad
                // glCullFace(GL_BACK); // TODO: support in miniquad
                // glFrontFace(GL_CCW); // DONE front_face_order

                // glEnable(GL_BLEND); // TODO_BELOW
                // glDisable(GL_DEPTH_TEST); DONE: depth_write: false, on PipelineParams
                // glDisable(GL_SCISSOR_TEST); // TODO: support in miniquad

                // glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE); // DONE color_write
                // glStencilMask(0xffffffff); // TODO: support in miniquad
                // glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP); // TODO: support in miniquad
                // glStencilFunc(GL_ALWAYS, 0, 0xffffffff); // TODO: support in miniquad

                // glActiveTexture(GL_TEXTURE0); // TODO: implement
                // glBindTexture(GL_TEXTURE_2D, 0); // TODO: implement

                // TODOKOLA: commented:
                // glBindVertexArray(self.vert_arr);
                // glBindBuffer(GL_ARRAY_BUFFER, self.vert_buf);
                // glBufferData(
                //     GL_ARRAY_BUFFER,
                //     (self.vertexes.len() * std::mem::size_of::<Vertex>()) as GLsizeiptr,
                //     self.vertexes.as_ptr() as *const c_void,
                //     GL_STREAM_DRAW,
                // );
                // glEnableVertexAttribArray(self.shader.loc_vertex);
                // glEnableVertexAttribArray(self.shader.loc_tcoord);
                // glVertexAttribPointer(
                //     self.shader.loc_vertex,
                //     2, // size in floats
                //     GL_FLOAT,
                //     GL_FALSE as GLboolean,
                //     std::mem::size_of::<Vertex>() as i32,
                //     std::ptr::null(),
                // );
                // glVertexAttribPointer(
                //     self.shader.loc_tcoord,
                //     2, // size in floats
                //     GL_FLOAT,
                //     GL_FALSE as GLboolean,
                //     std::mem::size_of::<Vertex>() as i32,
                //     (2 * std::mem::size_of::<f32>()) as *const c_void, // use GL_ARRAY_BUFFER, and skip x,y (2 floats) to start sampling at u, v
                // );
                // glUniform1i(self.shader.loc_tex, 0);
                // glUniform2fv(
                //     self.shader.loc_viewsize,
                //     1,
                //     &self.view as *const Extent as *const f32,
                // );

                let calls = &self.calls[..];
                for call in calls {
                    let blend = &call.blend_func;

                    self.ctx.set_blend(Some(blend.0));

                    // {
                    //     // TODO: set image in a better way!!!
                    //     self.bindings.images = vec![];
                    //     self.ctx.apply_bindings(&self.bindings);
                    // }

                    // glBlendFuncSeparate( // TODO: DELETE once tested
                    //     blend.src_rgb,
                    //     blend.dst_rgb,
                    //     blend.src_alpha,
                    //     blend.dst_alpha,
                    // );

                    match call.call_type {
                        CallType::Fill => self.do_fill(&call),
                        CallType::ConvexFill => self.do_convex_fill(&call),
                        CallType::Stroke => self.do_stroke(&call),
                        CallType::Triangles => {
                            // self.do_triangles(&call), WAS THIS
                            let uniforms = &self.uniforms[call.uniform_offset];
                            Self::set_uniforms(self.ctx, uniforms, call.image);

                            // TODO: update self.indices and self.vertexes
                            self.bindings.vertex_buffers[0].update(self.ctx, &self.indices[..]);
                            self.bindings.index_buffer.update(self.ctx, &self.indices[..]);

                            // glDrawArrays(
                            //     GL_TRIANGLES,
                            //     call.triangle_offset as i32,
                            //     call.triangle_count as i32,
                            // );
                    
                        }
                    }
                }

                self.ctx.end_render_pass();
                self.ctx.commit_frame();

                // TODO: commented, not needed??
                // glDisableVertexAttribArray(self.shader.loc_vertex);
                // glDisableVertexAttribArray(self.shader.loc_tcoord);
                // glBindVertexArray(0);
                // glDisable(GL_CULL_FACE);
                // glBindBuffer(GL_ARRAY_BUFFER, 0);
                // glUseProgram(0);
                // glBindTexture(GL_TEXTURE_2D, 0);
            }

        self.vertexes.clear();
        self.paths.clear();
        self.calls.clear();
        self.uniforms.clear();
        Ok(())
    }

    fn fill(
        &mut self,
        paint: &Paint,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
        fringe: f32,
        bounds: Bounds,
        paths: &[Path],
    ) -> anyhow::Result<()> {
        let mut call = Call {
            call_type: CallType::Fill,
            image: paint.image,
            path_offset: self.paths.len(),
            path_count: paths.len(),
            triangle_offset: 0,
            triangle_count: 4,
            uniform_offset: 0,
            blend_func: composite_operation.into(),
        };

        if paths.len() == 1 && paths[0].convex {
            call.call_type = CallType::ConvexFill;
        }

        let mut offset = self.vertexes.len();
        for path in paths {
            let fill = path.get_fill();
            let mut gl_path = GLPath {
                fill_offset: 0,
                fill_count: 0,
                stroke_offset: 0,
                stroke_count: 0,
            };

            if !fill.is_empty() {
                gl_path.fill_offset = offset;
                gl_path.fill_count = fill.len();
                self.vertexes.extend(fill);
                offset += fill.len();
            }

            let stroke = path.get_stroke();
            if !stroke.is_empty() {
                gl_path.stroke_offset = offset;
                gl_path.stroke_count = stroke.len();
                self.vertexes.extend(stroke);
                offset += stroke.len();
            }

            self.paths.push(gl_path);
        }

        if call.call_type == CallType::Fill {
            call.triangle_offset = offset;
            self.vertexes
                .push(Vertex::new(bounds.max.x, bounds.max.y, 0.5, 1.0));
            self.vertexes
                .push(Vertex::new(bounds.max.x, bounds.min.y, 0.5, 1.0));
            self.vertexes
                .push(Vertex::new(bounds.min.x, bounds.max.y, 0.5, 1.0));
            self.vertexes
                .push(Vertex::new(bounds.min.x, bounds.min.y, 0.5, 1.0));

            call.uniform_offset = self.uniforms.len();
            self.append_uniforms(shader::FragUniforms {
                stroke_thr: -1.0,
                type_: ShaderType::Simple as i32,
                ..shader::FragUniforms::default()
            });
            self.append_uniforms(self.convert_paint(paint, scissor, fringe, fringe, -1.0));
        } else {
            call.uniform_offset = self.uniforms.len();
            self.append_uniforms(self.convert_paint(paint, scissor, fringe, fringe, -1.0));
        }

        self.calls.push(call);
        Ok(())
    }

    fn stroke(
        &mut self,
        paint: &Paint,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[Path],
    ) -> anyhow::Result<()> {
        let mut call = Call {
            call_type: CallType::Stroke,
            image: paint.image,
            path_offset: self.paths.len(),
            path_count: paths.len(),
            triangle_offset: 0,
            triangle_count: 0,
            uniform_offset: 0,
            blend_func: composite_operation.into(),
        };

        let mut offset = self.vertexes.len();
        for path in paths {
            let mut gl_path = GLPath {
                fill_offset: 0,
                fill_count: 0,
                stroke_offset: 0,
                stroke_count: 0,
            };

            let stroke = path.get_stroke();
            if !stroke.is_empty() {
                gl_path.stroke_offset = offset;
                gl_path.stroke_count = stroke.len();
                self.vertexes.extend(stroke);
                offset += stroke.len();
                self.paths.push(gl_path);
            }
        }

        call.uniform_offset = self.uniforms.len();
        self.append_uniforms(self.convert_paint(paint, scissor, stroke_width, fringe, -1.0));
        self.append_uniforms(self.convert_paint(
            paint,
            scissor,
            stroke_width,
            fringe,
            1.0 - 0.5 / 255.0,
        ));

        self.calls.push(call);
        Ok(())
    }

    fn triangles(
        &mut self,
        paint: &Paint,
        composite_operation: CompositeOperationState,
        scissor: &Scissor,
        vertexes: &[Vertex],
    ) -> anyhow::Result<()> {
        let call = Call {
            call_type: CallType::Triangles,
            image: paint.image,
            path_offset: 0,
            path_count: 0,
            triangle_offset: self.vertexes.len(),
            triangle_count: vertexes.len(),
            uniform_offset: self.uniforms.len(),
            blend_func: composite_operation.into(),
        };

        self.calls.push(call);
        self.vertexes.extend(vertexes);

        let mut uniforms = self.convert_paint(paint, scissor, 1.0, 1.0, -1.0);
        uniforms.type_ = ShaderType::Image as i32;
        self.append_uniforms(uniforms);
        Ok(())
    }
}

fn convert_blend_factor(factor: nvg::BlendFactor) -> miniquad::BlendFactor {
    match factor {
        nvg::BlendFactor::Zero => miniquad::BlendFactor::Zero,
        nvg::BlendFactor::One => miniquad::BlendFactor::One,

        nvg::BlendFactor::SrcColor => miniquad::BlendFactor::Value(BlendValue::SourceColor),
        nvg::BlendFactor::OneMinusSrcColor => miniquad::BlendFactor::OneMinusValue(BlendValue::SourceColor),
        nvg::BlendFactor::DstColor => miniquad::BlendFactor::Value(BlendValue::DestinationColor),
        nvg::BlendFactor::OneMinusDstColor => miniquad::BlendFactor::OneMinusValue(BlendValue::DestinationColor),

        nvg::BlendFactor::SrcAlpha => miniquad::BlendFactor::Value(BlendValue::SourceAlpha),
        nvg::BlendFactor::OneMinusSrcAlpha => miniquad::BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        nvg::BlendFactor::DstAlpha => miniquad::BlendFactor::Value(BlendValue::DestinationAlpha),
        nvg::BlendFactor::OneMinusDstAlpha => miniquad::BlendFactor::OneMinusValue(BlendValue::DestinationAlpha),

        nvg::BlendFactor::SrcAlphaSaturate => miniquad::BlendFactor::SourceAlphaSaturate,
    }
}

#[inline]
fn premul_color(color: Color) -> Color {
    Color {
        r: color.r * color.a,
        g: color.g * color.a,
        b: color.b * color.a,
        a: color.a,
    }
}

#[inline]
fn xform_to_3x4(xform: Transform) -> [f32; 12] { // 3 col 4 rows
    let mut m = [0f32; 12];
    let t = &xform.0;
    m[0] = t[0];
    m[1] = t[1];
    m[2] = 0.0;
    m[3] = 0.0;
    m[4] = t[2];
    m[5] = t[3];
    m[6] = 0.0;
    m[7] = 0.0;
    m[8] = t[4];
    m[9] = t[5];
    m[10] = 1.0;
    m[11] = 0.0;
    m
}

#[inline]
fn xform_to_4x4(xform: Transform) -> Mat4 {
    let t = &xform.0;

    Mat4::from_cols(
        Vec4::new(t[0], t[2], t[4], 0.0), 
        Vec4::new(t[1], t[3], t[5], 0.0),
        Vec4::new(0.0, 0.0, 1.0, 0.0),
        Vec4::new(0.0, 0.0, 0.0, 0.0)
    )
}

