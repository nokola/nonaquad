use nvg::renderer::*;
use slab::Slab;
use std::ffi::c_void;
use miniquad::sapp::*;

struct Shader {
    prog: GLuint,
    frag: GLuint,
    vert: GLuint,
    loc_viewsize: i32,
    loc_tex: i32,
    loc_vertex: u32,
    loc_tcoord: u32,
    loc_scissor_mat: i32,
    loc_paint_mat: i32,
    loc_inner_col: i32,
    loc_outer_col: i32,
    loc_scissor_ext: i32,
    loc_scissor_scale: i32,
    loc_extent: i32,
    loc_radius: i32,
    loc_feather: i32,
    loc_stroke_mult: i32,
    loc_stroke_thr: i32,
    loc_tex_type: i32,
    loc_type: i32,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            // glDeleteProgram(self.prog); // TODO
            glDeleteShader(self.vert);
            glDeleteShader(self.frag);
        }
    }
}

impl Shader {
    unsafe fn load() -> anyhow::Result<Shader> {
        let mut status: GLint = std::mem::zeroed();
        let prog = glCreateProgram();
        let vert = glCreateShader(GL_VERTEX_SHADER);
        let frag = glCreateShader(GL_FRAGMENT_SHADER);
        let vert_source =
            std::ffi::CString::from_vec_unchecked(include_bytes!("shader.vert").to_vec());
        let frag_source =
            std::ffi::CString::from_vec_unchecked(include_bytes!("shader.frag").to_vec());

        glShaderSource(
            vert,
            1,
            [vert_source.as_ptr()].as_ptr() as *const *const GLchar,
            std::ptr::null(),
        );
        glShaderSource(
            frag,
            1,
            [frag_source.as_ptr()].as_ptr() as *const *const GLchar,
            std::ptr::null(),
        );

        glCompileShader(vert);
        glGetShaderiv(vert, GL_COMPILE_STATUS, &mut status);
        if status != GL_TRUE as i32 {
            return Err(shader_error(vert, "shader.vert"));
        }

        glCompileShader(frag);
        glGetShaderiv(frag, GL_COMPILE_STATUS, &mut status);
        if status != GL_TRUE as i32 {
            return Err(shader_error(vert, "shader.frag"));
        }

        glAttachShader(prog, vert);
        glAttachShader(prog, frag);

        // let name_vertex = std::ffi::CString::new("vertex").unwrap();
        // let name_tcoord = std::ffi::CString::new("tcoord").unwrap();
        // glBindAttribLocation(prog, 0, name_vertex.as_ptr() as *const i8); // TODO_INFO: commented out since unsupported for linking on Windows
        // glBindAttribLocation(prog, 1, name_tcoord.as_ptr() as *const i8); // TODO_INFO: commented out since unsupported for linking on Windows

        glLinkProgram(prog);
        glGetProgramiv(prog, GL_LINK_STATUS, &mut status);
        if status != GL_TRUE as i32 {
            return Err(program_error(prog));
        }

        let name_viewsize = std::ffi::CString::new("viewSize").unwrap();
        let name_tex = std::ffi::CString::new("tex").unwrap();

        Ok(Shader {
            prog,
            frag,
            vert,
            loc_viewsize: glGetUniformLocation(prog, name_viewsize.as_ptr() as *const GLchar),
            loc_tex: glGetUniformLocation(prog, name_tex.as_ptr() as *const GLchar),

            loc_vertex: glGetAttribLocation(prog, std::ffi::CString::new("vertex").unwrap().as_ptr() as *const GLchar) as u32,
            loc_tcoord: glGetAttribLocation(prog, std::ffi::CString::new("tcoord").unwrap().as_ptr() as *const GLchar) as u32,

            loc_scissor_mat: glGetUniformLocation(prog, std::ffi::CString::new("scissorMat").unwrap().as_ptr() as *const GLchar),
            loc_paint_mat: glGetUniformLocation(prog, std::ffi::CString::new("paintMat").unwrap().as_ptr() as *const GLchar),
            loc_inner_col: glGetUniformLocation(prog, std::ffi::CString::new("innerCol").unwrap().as_ptr() as *const GLchar),
            loc_outer_col: glGetUniformLocation(prog, std::ffi::CString::new("outerCol").unwrap().as_ptr() as *const GLchar),
            loc_scissor_ext: glGetUniformLocation(prog, std::ffi::CString::new("scissorExt").unwrap().as_ptr() as *const GLchar),
            loc_scissor_scale: glGetUniformLocation(prog, std::ffi::CString::new("scissorScale").unwrap().as_ptr() as *const GLchar),
            loc_extent: glGetUniformLocation(prog, std::ffi::CString::new("extent").unwrap().as_ptr() as *const GLchar),
            loc_radius: glGetUniformLocation(prog, std::ffi::CString::new("radius").unwrap().as_ptr() as *const GLchar),
            loc_feather: glGetUniformLocation(prog, std::ffi::CString::new("feather").unwrap().as_ptr() as *const GLchar),
            loc_stroke_mult: glGetUniformLocation(prog, std::ffi::CString::new("strokeMult").unwrap().as_ptr() as *const GLchar),
            loc_stroke_thr: glGetUniformLocation(prog, std::ffi::CString::new("strokeThr").unwrap().as_ptr() as *const GLchar),
            loc_tex_type: glGetUniformLocation(prog, std::ffi::CString::new("texType").unwrap().as_ptr() as *const GLchar),
            loc_type: glGetUniformLocation(prog, std::ffi::CString::new("type").unwrap().as_ptr() as *const GLchar),
        })
    }
}

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

struct Blend {
    src_rgb: GLenum,
    dst_rgb: GLenum,
    src_alpha: GLenum,
    dst_alpha: GLenum,
}

impl From<CompositeOperationState> for Blend {
    fn from(state: CompositeOperationState) -> Self {
        Blend {
            src_rgb: convert_blend_factor(state.src_rgb),
            dst_rgb: convert_blend_factor(state.dst_rgb),
            src_alpha: convert_blend_factor(state.src_alpha),
            dst_alpha: convert_blend_factor(state.dst_alpha),
        }
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
    tex: GLuint,
    width: usize,
    height: usize,
    texture_type: TextureType,
    flags: ImageFlags,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { glDeleteTextures(1, &self.tex) }
    }
}

struct GLPath {
    fill_offset: usize,
    fill_count: usize,
    stroke_offset: usize,
    stroke_count: usize,
}

#[derive(Default)]
#[allow(dead_code)]
struct FragUniforms {
    scissor_mat: [f32; 12],
    paint_mat: [f32; 12],
    inner_color: Color,
    outer_color: Color,
    scissor_ext: [f32; 2],
    scissor_scale: [f32; 2],
    extent: [f32; 2],
    radius: f32,
    feather: f32,
    stroke_mult: f32,
    stroke_thr: f32,
    tex_type: i32,
    type_: i32,
}

pub struct Renderer {
    shader: Shader,
    textures: Slab<Texture>,
    view: Extent,
    vert_buf: GLuint,
    vert_arr: GLuint,
    calls: Vec<Call>,
    paths: Vec<GLPath>,
    vertexes: Vec<Vertex>,
    uniforms: Vec<FragUniforms>,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            // glDeleteBuffers(1, &self.frag_buf); TODOKOLA
            glDeleteBuffers(1, &self.vert_buf);
            // glDeleteVertexArrays(1, &self.vert_arr); TODOKOLA
        }
    }
}

impl Renderer {
    pub fn create() -> anyhow::Result<Renderer> {
        unsafe {
            let shader = Shader::load()?;

            let mut vert_arr: GLuint = std::mem::zeroed();
            glGenVertexArrays(1, &mut vert_arr);

            let mut vert_buf: GLuint = std::mem::zeroed();
            glGenBuffers(1, &mut vert_buf);

            // glUniformBlockBinding(shader.prog, shader.loc_frag, 0);
            // let mut frag_buf: GLuint = std::mem::zeroed();
            // glGenBuffers(1, &mut frag_buf);

            // let mut align = std::mem::zeroed();
            // glGetIntegerv(GL_UNIFORM_BUFFER_OFFSET_ALIGNMENT, &mut align);

            // let frag_size = std::mem::size_of::<FragUniforms>() + (align as usize)
            //     - std::mem::size_of::<FragUniforms>() % (align as usize);

            // glFinish();

            Ok(Renderer {
                shader,
                textures: Default::default(),
                view: Default::default(),
                vert_buf,
                vert_arr,
                calls: Default::default(),
                paths: Default::default(),
                vertexes: Default::default(),
                uniforms: Default::default(),
            })
        }
    }

    unsafe fn set_uniforms(&self, offset: usize, img: Option<usize>) {
        let uniforms = &self.uniforms[offset];
        // glBindBuffer(GL_UNIFORM_BUFFER, self.frag_buf); // TODOKOLA: added
        glUniformMatrix4fv(self.shader.loc_scissor_mat, 1, 0, &uniforms.scissor_mat as *const _ as *const f32);
        glUniformMatrix4fv(self.shader.loc_paint_mat, 1, 0, &uniforms.paint_mat as *const _ as *const f32);

        glUniform4fv(self.shader.loc_inner_col, 1, &uniforms.inner_color as *const _ as *const f32);
        glUniform4fv(self.shader.loc_outer_col, 1, &uniforms.outer_color as *const _ as *const f32);

        glUniform2fv(self.shader.loc_scissor_ext, 1, &uniforms.scissor_ext as *const _ as *const f32);
        glUniform2fv(self.shader.loc_scissor_scale, 1, &uniforms.scissor_scale as *const _ as *const f32);
        glUniform2fv(self.shader.loc_extent, 1, &uniforms.extent as *const _ as *const f32);

        glUniform1f(self.shader.loc_radius, uniforms.radius);
        glUniform1f(self.shader.loc_feather, uniforms.feather);
        glUniform1f(self.shader.loc_stroke_mult, uniforms.stroke_mult);
        glUniform1f(self.shader.loc_stroke_thr, uniforms.stroke_thr);

        glUniform1i(self.shader.loc_tex_type, uniforms.tex_type);
        glUniform1i(self.shader.loc_type, uniforms.type_);

        // glBindBufferRange( // TODOKOLA
        //     GL_UNIFORM_BUFFER,
        //     0,
        //     self.frag_buf,
        //     (offset * self.frag_size) as isize,
        //     std::mem::size_of::<FragUniforms>() as GLsizeiptr,
        // );

        if let Some(img) = img {
            if let Some(texture) = self.textures.get(img) {
                glBindTexture(GL_TEXTURE_2D, texture.tex);
            }
        } else {
            glBindTexture(GL_TEXTURE_2D, 0);
        }
    }

    unsafe fn do_fill(&self, call: &Call) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];

        glEnable(GL_STENCIL_TEST);
        glStencilMask(0xff);
        glStencilFunc(GL_ALWAYS, 0, 0xff);
        glColorMask(GL_FALSE, GL_FALSE, GL_FALSE, GL_FALSE);

        self.set_uniforms(call.uniform_offset, call.image);

        // glStencilOpSeparate(GL_FRONT, GL_KEEP, GL_KEEP, GL_INCR_WRAP);
        // glStencilOpSeparate(GL_BACK, GL_KEEP, GL_KEEP, GL_DECR_WRAP);
        glDisable(GL_CULL_FACE);
        for path in paths {
            glDrawArrays(
                GL_TRIANGLE_FAN,
                path.fill_offset as i32,
                path.fill_count as i32,
            );
        }
        glEnable(GL_CULL_FACE);

        glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);

        self.set_uniforms(call.uniform_offset + 1, call.image);

        glStencilFunc(GL_EQUAL, 0x00, 0xff);
        glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);
        for path in paths {
            glDrawArrays(
                GL_TRIANGLE_STRIP,
                path.stroke_offset as i32,
                path.stroke_count as i32,
            );
        }

        glStencilFunc(GL_NOTEQUAL, 0x00, 0xff);
        glStencilOp(GL_ZERO, GL_ZERO, GL_ZERO);
        glDrawArrays(
            GL_TRIANGLE_STRIP,
            call.triangle_offset as i32,
            call.triangle_count as i32,
        );

        glDisable(GL_STENCIL_TEST);
    }

    unsafe fn do_convex_fill(&self, call: &Call) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];
        self.set_uniforms(call.uniform_offset, call.image);
        for path in paths {
            glDrawArrays(
                GL_TRIANGLE_FAN,
                path.fill_offset as i32,
                path.fill_count as i32,
            );
            if path.stroke_count > 0 {
                glDrawArrays(
                    GL_TRIANGLE_STRIP,
                    path.stroke_offset as i32,
                    path.stroke_count as i32,
                );
            }
        }
    }

    unsafe fn do_stroke(&self, call: &Call) {
        let paths = &self.paths[call.path_offset..call.path_offset + call.path_count];

        glEnable(GL_STENCIL_TEST);
        glStencilMask(0xff);
        glStencilFunc(GL_EQUAL, 0x0, 0xff);
        glStencilOp(GL_KEEP, GL_KEEP, GL_INCR);
        self.set_uniforms(call.uniform_offset + 1, call.image);
        for path in paths {
            glDrawArrays(
                GL_TRIANGLE_STRIP,
                path.stroke_offset as i32,
                path.stroke_count as i32,
            );
        }

        self.set_uniforms(call.uniform_offset, call.image);
        glStencilFunc(GL_EQUAL, 0x0, 0xff);
        glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);
        for path in paths {
            glDrawArrays(
                GL_TRIANGLE_STRIP,
                path.stroke_offset as i32,
                path.stroke_count as i32,
            );
        }

        glColorMask(GL_FALSE, GL_FALSE, GL_FALSE, GL_FALSE);
        glStencilFunc(GL_ALWAYS, 0x0, 0xff);
        glStencilOp(GL_ZERO, GL_ZERO, GL_ZERO);
        for path in paths {
            glDrawArrays(
                GL_TRIANGLE_STRIP,
                path.stroke_offset as i32,
                path.stroke_count as i32,
            );
        }
        glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);

        glDisable(GL_STENCIL_TEST);
    }

    unsafe fn do_triangles(&self, call: &Call) {
        self.set_uniforms(call.uniform_offset, call.image);
        glDrawArrays(
            GL_TRIANGLES,
            call.triangle_offset as i32,
            call.triangle_count as i32,
        );
    }

    fn convert_paint(
        &self,
        paint: &Paint,
        scissor: &Scissor,
        width: f32,
        fringe: f32,
        stroke_thr: f32,
    ) -> FragUniforms {
        let mut frag = FragUniforms {
            scissor_mat: Default::default(),
            paint_mat: Default::default(),
            inner_color: premul_color(paint.inner_color),
            outer_color: premul_color(paint.outer_color),
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
            frag.scissor_ext[0] = 1.0;
            frag.scissor_ext[1] = 1.0;
            frag.scissor_scale[0] = 1.0;
            frag.scissor_scale[1] = 1.0;
        } else {
            frag.scissor_mat = xform_to_3x4(scissor.xform.inverse());
            frag.scissor_ext[0] = scissor.extent.width;
            frag.scissor_ext[1] = scissor.extent.height;
            frag.scissor_scale[0] = (scissor.xform.0[0] * scissor.xform.0[0]
                + scissor.xform.0[2] * scissor.xform.0[2])
                .sqrt()
                / fringe;
            frag.scissor_scale[1] = (scissor.xform.0[1] * scissor.xform.0[1]
                + scissor.xform.0[3] * scissor.xform.0[3])
                .sqrt()
                / fringe;
        }

        frag.extent = [paint.extent.width, paint.extent.height];
        frag.stroke_mult = (width * 0.5 + fringe * 0.5) / fringe;

        let mut invxform = Transform::default();

        if let Some(img) = paint.image {
            if let Some(texture) = self.textures.get(img) {
                if texture.flags.contains(ImageFlags::FLIPY) {
                    let m1 = Transform::translate(0.0, frag.extent[1] * 0.5) * paint.xform;
                    let m2 = Transform::scale(1.0, -1.0) * m1;
                    let m1 = Transform::translate(0.0, -frag.extent[1] * 0.5) * m2;
                    invxform = m1.inverse();
                } else {
                    invxform = paint.xform.inverse();
                };

                frag.type_ = ShaderType::FillImage as i32;
                match texture.texture_type {
                    TextureType::RGBA => {
                        frag.tex_type = if texture.flags.contains(ImageFlags::PREMULTIPLIED) {
                            0
                        } else {
                            1
                        }
                    }
                    TextureType::Alpha => frag.tex_type = 2,
                }
            }
        } else {
            frag.type_ = ShaderType::FillGradient as i32;
            frag.radius = paint.radius;
            frag.feather = paint.feather;
            invxform = paint.xform.inverse();
        }

        frag.paint_mat = xform_to_3x4(invxform);

        frag
    }

    fn append_uniforms(&mut self, uniforms: FragUniforms) {
        self.uniforms.push(uniforms);
        //     .resize(self.uniforms.len() + self.frag_size, 0);
        // unsafe {
        //     let idx = self.uniforms.len() - self.frag_size;
        //     let p = self.uniforms.as_mut_ptr().add(idx) as *mut FragUniforms;
        //     *p = uniforms;
        // }
    }
}

impl renderer::Renderer for Renderer {
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
        let tex = unsafe {
            let mut tex: GLuint = std::mem::zeroed();
            glGenTextures(1, &mut tex);
            glBindTexture(GL_TEXTURE_2D, tex);
            // glPixelStorei(GL_UNPACK_ALIGNMENT, 1);TODOKOLA

            match texture_type {
                TextureType::RGBA => {
                    glTexImage2D(
                        GL_TEXTURE_2D,
                        0,
                        GL_RGBA as i32,
                        width as i32,
                        height as i32,
                        0,
                        GL_RGBA,
                        GL_UNSIGNED_BYTE,
                        match data {
                            Some(data) => data.as_ptr() as *const c_void,
                            Option::None => std::ptr::null(),
                        },
                    );
                }
                TextureType::Alpha => {
                    glTexImage2D(
                        GL_TEXTURE_2D,
                        0,
                        GL_R8 as i32,
                        width as i32,
                        height as i32,
                        0,
                        GL_RED,
                        GL_UNSIGNED_BYTE,
                        match data {
                            Some(data) => data.as_ptr() as *const c_void,
                            Option::None => std::ptr::null(),
                        },
                    );
                }
            }

            if flags.contains(ImageFlags::GENERATE_MIPMAPS) {
                if flags.contains(ImageFlags::NEAREST) {
                    glTexParameteri(
                        GL_TEXTURE_2D,
                        GL_TEXTURE_MIN_FILTER,
                        GL_NEAREST_MIPMAP_NEAREST as i32,
                    );
                } else {
                    glTexParameteri(
                        GL_TEXTURE_2D,
                        GL_TEXTURE_MIN_FILTER,
                        GL_LINEAR_MIPMAP_LINEAR as i32,
                    );
                }
            } else {
                if flags.contains(ImageFlags::NEAREST) {
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST as i32);
                } else {
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
                }
            }

            if flags.contains(ImageFlags::NEAREST) {
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST as i32);
            } else {
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
            }

            if flags.contains(ImageFlags::REPEATX) {
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32);
            } else {
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE as i32);
            }

            if flags.contains(ImageFlags::REPEATY) {
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
            } else {
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE as i32);
            }

            // glPixelStorei(GL_UNPACK_ALIGNMENT, 4); TODOKOLA

            // if flags.contains(ImageFlags::GENERATE_MIPMAPS) { TODOKOLA
            //     glGenerateMipmap(GL_TEXTURE_2D);
            // }

            glBindTexture(GL_TEXTURE_2D, 0);
            tex
        };

        let id = self.textures.insert(Texture {
            tex,
            width,
            height,
            texture_type,
            flags,
        });
        Ok(id)
    }

    fn delete_texture(&mut self, img: ImageId) -> anyhow::Result<()> {
        if let Some(texture) = self.textures.get(img) {
            unsafe { glDeleteTextures(1, &texture.tex) }
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
            unsafe {
                glBindTexture(GL_TEXTURE_2D, texture.tex);
                // glPixelStorei(GL_UNPACK_ALIGNMENT, 1); TODOKOLA

                match texture.texture_type {
                    TextureType::RGBA => glTexSubImage2D(
                        GL_TEXTURE_2D,
                        0,
                        x as i32,
                        y as i32,
                        width as i32,
                        height as i32,
                        GL_RGBA,
                        GL_UNSIGNED_BYTE,
                        data.as_ptr() as *const c_void,
                    ),
                    TextureType::Alpha => glTexSubImage2D(
                        GL_TEXTURE_2D,
                        0,
                        x as i32,
                        y as i32,
                        width as i32,
                        height as i32,
                        GL_RED,
                        GL_UNSIGNED_BYTE,
                        data.as_ptr() as *const c_void,
                    ),
                }

                // glPixelStorei(GL_UNPACK_ALIGNMENT, 4); TODOKOLA
                glBindTexture(GL_TEXTURE_2D, 0);
            }
            Ok(())
        } else {
            bail!("texture '{}' not found", img);
        }
    }

    fn texture_size(&self, img: ImageId) -> anyhow::Result<(usize, usize)> {
        if let Some(texture) = self.textures.get(img) {
            Ok((texture.width, texture.height))
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
        if !self.calls.is_empty() {
            unsafe {
                glUseProgram(self.shader.prog);

                glEnable(GL_CULL_FACE);
                glCullFace(GL_BACK);
                glFrontFace(GL_CCW);
                glEnable(GL_BLEND);
                glDisable(GL_DEPTH_TEST);
                glDisable(GL_SCISSOR_TEST);
                glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);
                glStencilMask(0xffffffff);
                glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);
                glStencilFunc(GL_ALWAYS, 0, 0xffffffff);
                glActiveTexture(GL_TEXTURE0);
                glBindTexture(GL_TEXTURE_2D, 0);

                // glBindBuffer(GL_UNIFORM_BUFFER, self.frag_buf); TODOKOLA commented
                // glBufferData(
                //     GL_UNIFORM_BUFFER,
                //     self.uniforms.len() as GLsizeiptr,
                //     self.uniforms.as_ptr() as *const c_void,
                //     GL_STREAM_DRAW,
                // );

                glBindVertexArray(self.vert_arr);
                glBindBuffer(GL_ARRAY_BUFFER, self.vert_buf);
                glBufferData(
                    GL_ARRAY_BUFFER,
                    (self.vertexes.len() * std::mem::size_of::<Vertex>()) as GLsizeiptr,
                    self.vertexes.as_ptr() as *const c_void,
                    GL_STREAM_DRAW,
                );
                glEnableVertexAttribArray(self.shader.loc_vertex);
                glEnableVertexAttribArray(self.shader.loc_tcoord);
                glVertexAttribPointer(
                    self.shader.loc_vertex,
                    2, // size in floats
                    GL_FLOAT,
                    GL_FALSE as GLboolean,
                    std::mem::size_of::<Vertex>() as i32,
                    std::ptr::null(),
                );
                glVertexAttribPointer(
                    self.shader.loc_tcoord,
                    2, // size in floats
                    GL_FLOAT,
                    GL_FALSE as GLboolean,
                    std::mem::size_of::<Vertex>() as i32,
                    (2 * std::mem::size_of::<f32>()) as *const c_void, // use GL_ARRAY_BUFFER, and skip x,y (2 floats) to start sampling at u, v
                );

                glUniform1i(self.shader.loc_tex, 0);
                glUniform2fv(
                    self.shader.loc_viewsize,
                    1,
                    &self.view as *const Extent as *const f32,
                );

                // glBindBuffer(GL_UNIFORM_BUFFER, self.frag_buf); TODOKOLA

                for call in &self.calls {
                    let blend = &call.blend_func;

                    glBlendFuncSeparate(
                        blend.src_rgb,
                        blend.dst_rgb,
                        blend.src_alpha,
                        blend.dst_alpha,
                    );

                    match call.call_type {
                        CallType::Fill => self.do_fill(&call),
                        CallType::ConvexFill => self.do_convex_fill(&call),
                        CallType::Stroke => self.do_stroke(&call),
                        CallType::Triangles => self.do_triangles(&call),
                    }
                }

                glDisableVertexAttribArray(self.shader.loc_vertex);
                glDisableVertexAttribArray(self.shader.loc_tcoord);
                glBindVertexArray(0);
                glDisable(GL_CULL_FACE);
                glBindBuffer(GL_ARRAY_BUFFER, 0);
                glUseProgram(0);
                glBindTexture(GL_TEXTURE_2D, 0);
            }
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
            self.append_uniforms(FragUniforms {
                stroke_thr: -1.0,
                type_: ShaderType::Simple as i32,
                ..FragUniforms::default()
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

fn shader_error(shader: GLuint, filename: &str) -> anyhow::Error {
    unsafe {
        let mut data: [GLchar; 512 + 1] = std::mem::zeroed();
        let mut len: GLsizei = std::mem::zeroed();
        glGetShaderInfoLog(shader, 512, &mut len, data.as_mut_ptr());
        if len > 512 {
            len = 512;
        }
        data[len as usize] = 0;
        let err_msg = std::ffi::CStr::from_ptr(data.as_ptr());
        anyhow!(
            "failed to compile shader: {}: {}",
            filename,
            err_msg.to_string_lossy()
        )
    }
}

fn program_error(prog: GLuint) -> anyhow::Error {
    unsafe {
        let mut data: [GLchar; 512 + 1] = std::mem::zeroed();
        let mut len: GLsizei = std::mem::zeroed();
        glGetProgramInfoLog(prog, 512, &mut len, data.as_mut_ptr());
        if len > 512 {
            len = 512;
        }
        data[len as usize] = 0;
        let err_msg = std::ffi::CStr::from_ptr(data.as_ptr());
        anyhow!("failed to link program: {}", err_msg.to_string_lossy())
    }
}

fn convert_blend_factor(factor: BlendFactor) -> GLenum {
    match factor {
        BlendFactor::Zero => GL_ZERO,
        BlendFactor::One => GL_ONE,
        BlendFactor::SrcColor => GL_SRC_COLOR,
        BlendFactor::OneMinusSrcColor => GL_ONE_MINUS_SRC_COLOR,
        BlendFactor::DstColor => GL_DST_COLOR,
        BlendFactor::OneMinusDstColor => GL_ONE_MINUS_DST_COLOR,
        BlendFactor::SrcAlpha => GL_SRC_ALPHA,
        BlendFactor::OneMinusSrcAlpha => GL_ONE_MINUS_SRC_ALPHA,
        BlendFactor::DstAlpha => GL_DST_ALPHA,
        BlendFactor::OneMinusDstAlpha => GL_ONE_MINUS_DST_ALPHA,
        BlendFactor::SrcAlphaSaturate => GL_SRC_ALPHA_SATURATE,
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
fn xform_to_3x4(xform: Transform) -> [f32; 12] {
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
