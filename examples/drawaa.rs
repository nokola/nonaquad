use miniquad::*;
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
}

static mut MINI_CONTEXT: Option<Context> = None;
fn get_context() -> &'static mut Context {
    unsafe { MINI_CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

static mut NVG_CONTEXT: Option<nvg::Context<nvgimpl::Renderer>> = None;
fn get_nvg_context() -> &'static mut nvg::Context<nvgimpl::Renderer<'static>> {
    unsafe { NVG_CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

impl Stage {
    pub fn new() -> Stage {
        let renderer = nvgimpl::Renderer::create(get_context()).unwrap();
        let nvg_context = nvg::Context::create(renderer).unwrap();
        unsafe { NVG_CONTEXT = Some(nvg_context) };

        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertex_buffer = Buffer::immutable(get_context(), BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(get_context(), BufferType::IndexBuffer, &indices);

        let pixels: [u8; 4 * 4 * 4] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
            0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];
        let texture = Texture::from_rgba8(get_context(), 4, 4, &pixels);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![texture],
        };

        let shader = Shader::new(
            get_context(),
            shader::VERTEX,
            shader::FRAGMENT,
            shader::META,
        );

        let pipeline = Pipeline::new(
            get_context(),
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
            // nvg_context,
        }
    }
}

impl EventHandlerFree for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        // let t = date::now();

        let ctx = get_context();

        let (width, height) = ctx.screen_size();
        let device_pixel_ratio = ctx.dpi_scale();
        // ctx.begin_default_pass(Default::default()); // comment to show the glClear!!; bind framebuffer, viewport, scissor

        // glViewport(0, 0, width as i32, height as i32); // needed for WebGL to render correctly to canvas
        // glScissor(0, 0, width as i32, height as i32);
        // glClearColor(0.5, 0.2, 0.5, 1.0); // PURPLE
        // glClear(GL_COLOR_BUFFER_BIT);

        let nvg_context = get_nvg_context();

        // let renderer = nvg_context
        nvg_context
            .begin_frame(
                nvg::Extent {
                    width: width as f32,
                    height: height as f32,
                },
                device_pixel_ratio,
            )
            .unwrap();
        nvg_context.begin_path();
        nvg_context.rect((100.0, 100.0, 300.0, 300.0));
        nvg_context.fill_paint(nvg::Gradient::Linear {
            start: (100, 100).into(),
            end: (400, 400).into(),
            start_color: nvg::Color::rgb_i(0xAA, 0x6C, 0x39),
            end_color: nvg::Color::rgb_i(0x88, 0x2D, 0x60),
        });
        nvg_context.fill().unwrap();

        // let origin = (150.0, 140.0);
        // nvg_context.begin_path();
        // // nvg_context.shape_antialias(false);
        // nvg_context.circle(origin, 64.0);
        // nvg_context.move_to(origin);
        // nvg_context.line_to((origin.0 + 300.0, origin.1 - 50.0));
        // nvg_context.stroke_paint(nvg::Color::rgba(1.0, 1.0, 0.0, 1.0));
        // nvg_context.stroke_width(3.0);
        // nvg_context.stroke().unwrap();

        // nvg_context.save();
        // // nvg_context.global_composite_operation(nvg::CompositeOperation::Basic(nvg::BasicCompositeOperation::Lighter));
        // let origin = (150.0, 140.0);
        // nvg_context.begin_path();
        // nvg_context.circle(origin, 64.0);
        // nvg_context.move_to(origin);
        // nvg_context.line_join(nvg::LineJoin::Round);
        // nvg_context.line_to((origin.0 + 300.0, origin.1 - 50.0));
        // nvg_context.quad_to((300.0, 100.0), (origin.0 + 500.0, origin.1 + 100.0));
        // nvg_context.close_path();
        // nvg_context.fill_paint(nvg::Color::rgba(0.2, 0.0, 0.8, 1.0));
        // nvg_context.fill().unwrap();
        // nvg_context.stroke_paint(nvg::Color::rgba(1.0, 1.0, 0.0, 1.0));
        // nvg_context.stroke_width(3.0);
        // nvg_context.stroke().unwrap();
        // nvg_context.restore();

        nvg_context.end_frame().unwrap(); // comment to show stuff

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
    color_backtrace::install();

    miniquad::start(conf::Conf::default(), |ctx| {
        unsafe { MINI_CONTEXT = Some(ctx) };

        UserData::free(Stage::new())
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
            uniforms: &[UniformDesc::new("offset", UniformType::Float2)],
        },
    };

    #[repr(C)]
    pub struct Uniforms {
        pub offset: (f32, f32),
    }
}
