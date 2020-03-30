use miniquad::*;
use miniquad::sapp::*;
use nvg_miniquad::nvgimpl;

#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}
#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

struct Stage {
    pipeline: Pipeline,
    bindings: Bindings,
    nvg_context: nvg::Context<nvgimpl::Renderer>,
}

impl Stage {
    pub fn new(ctx: &mut Context) -> Stage {
        let renderer1 = nvgimpl::Renderer::create().unwrap();
        let nvg_context = nvg::Context::create(renderer1).unwrap();

        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        let pixels: [u8; 4 * 4 * 4] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
            0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let texture = Texture::from_rgba8(ctx, 4, 4, &pixels);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![texture],
        };

        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::META);

        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
        );

        Stage {
            pipeline,
            bindings,
            nvg_context,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        // let t = date::now();

        unsafe {
            glClearColor(0.5, 0.5, 0.2, 1.0); // YELLOW
            glClear(GL_COLOR_BUFFER_BIT);
        }
        let (width, height) = ctx.screen_size();
        let device_pixel_ratio = ctx.dpi_scale();

        // ctx.begin_default_pass(Default::default()); // comment to show the glClear!!; bind framebuffer, viewport, scissor
        
        unsafe {
            glViewport(0, 0, width as i32, height as i32); // needed for WebGL to render correctly to canvas
            glScissor(0, 0, width as i32, height as i32);
        }

        unsafe {
            glClearColor(0.5, 0.2, 0.5, 1.0); // PURPLE
            glClear(GL_COLOR_BUFFER_BIT);
        }


        // let renderer = self.nvg_context
        self.nvg_context.begin_frame(
                nvg::Extent {
                    width: width as f32,
                    height: height as f32,
                },
                device_pixel_ratio,
            )
            .unwrap();
    
        self.nvg_context.begin_path();
        self.nvg_context.rect((100.0, 100.0, 300.0, 300.0));
        self.nvg_context.fill_paint(nvg::Gradient::Linear {
            start: (100, 100).into(),
            end: (400, 400).into(),
            start_color: nvg::Color::rgb_i(0xAA, 0x6C, 0x39),
            end_color: nvg::Color::rgb_i(0x88, 0x2D, 0x60),
        });
        self.nvg_context.fill().unwrap();

        let origin = (150.0, 140.0);
        self.nvg_context.begin_path();
        // self.nvg_context.shape_antialias(false);
        self.nvg_context.circle(origin, 64.0);
        self.nvg_context.move_to(origin);
        self.nvg_context.line_to((origin.0 + 300.0, origin.1 - 50.0));
        self.nvg_context.stroke_paint(nvg::Color::rgba(1.0, 1.0, 0.0, 1.0));
        self.nvg_context.stroke_width(3.0);
        self.nvg_context.stroke().unwrap();

        // self.nvg_context.save();
        // // self.nvg_context.global_composite_operation(nvg::CompositeOperation::Basic(nvg::BasicCompositeOperation::Lighter));
        // let origin = (150.0, 140.0);
        // self.nvg_context.begin_path();
        // self.nvg_context.circle(origin, 64.0);
        // self.nvg_context.move_to(origin);
        // self.nvg_context.line_join(nvg::LineJoin::Round);
        // self.nvg_context.line_to((origin.0 + 300.0, origin.1 - 50.0));
        // self.nvg_context.quad_to((300.0, 100.0), (origin.0 + 500.0, origin.1 + 100.0));
        // self.nvg_context.close_path();
        // self.nvg_context.fill_paint(nvg::Color::rgba(0.2, 0.0, 0.8, 1.0));
        // self.nvg_context.fill().unwrap();
        // self.nvg_context.stroke_paint(nvg::Color::rgba(1.0, 1.0, 0.0, 1.0));
        // self.nvg_context.stroke_width(3.0);
        // self.nvg_context.stroke().unwrap();
        // self.nvg_context.restore();


        self.nvg_context.end_frame().unwrap(); // comment to show stuff

        // ctx.apply_pipeline(&self.pipeline);
        // ctx.apply_bindings(&self.bindings);
        // for i in 0..10 {
        //     let t = t + i as f64 * 0.3;

        //     ctx.apply_uniforms(&shader::Uniforms {
        //         offset: (t.sin() as f32 * 0.5, (t * 3.).cos() as f32 * 0.5),
        //     });
        //     ctx.draw(0, 6, 1);
        // }
        // ctx.end_render_pass();

        // ctx.commit_frame(); // will call clear_buffer_bindings() and clear_texture_bindings()
    }
}

fn main() {
    // color_backtrace::install();

    miniquad::start(conf::Conf::default(), |mut ctx| {
        UserData::owning(Stage::new(&mut ctx), ctx)
    });
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 uv;

    uniform vec2 offset;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(pos + offset, 0, 1);
        texcoord = uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, texcoord);
    }"#;

    pub const META: ShaderMeta = ShaderMeta {
        images: &["tex"],
        uniforms: UniformBlockLayout {
            uniforms: &[("offset", UniformType::Float2)],
        },
    };

    #[repr(C)]
    pub struct Uniforms {
        pub offset: (f32, f32),
    }
}
