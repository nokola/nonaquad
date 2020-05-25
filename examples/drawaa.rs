use miniquad::*;
use nvg_miniquad::nvgimpl;
// use nvg_miniquad::nvgimpl_orig as nvgimpl;

struct Stage {}

static mut MINI_CONTEXT: Option<Context> = None;
fn get_context() -> &'static mut Context {
    unsafe { MINI_CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

static mut NVG_CONTEXT: Option<nona::Context<nvgimpl::Renderer>> = None;
fn get_nvg_context() -> &'static mut nona::Context<nvgimpl::Renderer<'static>> {
    unsafe { NVG_CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

impl Stage {
    pub fn new() -> Stage {
        let renderer = nvgimpl::Renderer::create(get_context()).unwrap();
        let mut nvg_context = nona::Context::create(renderer).unwrap();
        nvg_context.create_font_from_file("roboto", "examples/Roboto-Bold.ttf").unwrap();
        unsafe { NVG_CONTEXT = Some(nvg_context) };
        Stage {}
    }
}

impl EventHandlerFree for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        let ctx = get_context();

        let (width, height) = ctx.screen_size(); // the <physical width> == <logical width> * ctx.dpi_scale()
        let device_pixel_ratio = ctx.dpi_scale(); // e.g. 1.5 for 150% scale
        let nvg_context = get_nvg_context();

        nvg_context
            .begin_frame(
                nona::Extent {
                    width: width as f32,
                    height: height as f32,
                },
                device_pixel_ratio,
            )
            .unwrap();
        nvg_context.begin_path();
        nvg_context.rect((100.0, 100.0, 400.0, 300.0));
        nvg_context.fill_paint(nona::Gradient::Linear {
            start: (100, 100).into(),
            end: (400, 400).into(),
            start_color: nona::Color::rgb_i(0xAA, 0x6C, 0x39),
            end_color: nona::Color::rgb_i(0x88, 0x2D, 0x60),
        });
        nvg_context.fill().unwrap();

        let origin = (150.0, 140.0);
        nvg_context.begin_path();
        nvg_context.circle(origin, 64.0);
        nvg_context.move_to(origin);
        nvg_context.line_to((origin.0 + 300.0, origin.1 - 50.0));
        nvg_context.stroke_paint(nona::Color::rgba(1.0, 1.0, 0.0, 1.0));
        nvg_context.stroke_width(3.0);
        nvg_context.stroke().unwrap();

        nvg_context.fill_paint(nona::Color::rgb(1.0, 1.0, 1.0));
        nvg_context.font("roboto");
        nvg_context.font_size(40.0);
        nvg_context.begin_path();
        nvg_context.text_align(nona::Align::TOP | nona::Align::LEFT);
        nvg_context.text((10, 10), format!("alpha texture font - working!!!")).unwrap();
        nvg_context.fill().unwrap();

        // nvg_context.save();
        // // nvg_context.global_composite_operation(nona::CompositeOperation::Basic(nona::BasicCompositeOperation::Lighter));
        // let origin = (150.0, 140.0);
        // nvg_context.begin_path();
        // nvg_context.circle(origin, 64.0);
        // nvg_context.move_to(origin);
        // nvg_context.line_join(nona::LineJoin::Round);
        // nvg_context.line_to((origin.0 + 300.0, origin.1 - 50.0));
        // nvg_context.quad_to((300.0, 100.0), (origin.0 + 500.0, origin.1 + 100.0));
        // nvg_context.close_path();
        // nvg_context.fill_paint(nona::Color::rgba(0.2, 0.0, 0.8, 1.0));
        // nvg_context.fill().unwrap();
        // nvg_context.stroke_paint(nona::Color::rgba(1.0, 1.0, 0.0, 1.0));
        // nvg_context.stroke_width(3.0);
        // nvg_context.stroke().unwrap();
        // nvg_context.restore();

        nvg_context.end_frame().unwrap();
    }
}

fn main() {
    // color_backtrace::install();

    miniquad::start(conf::Conf {
     high_dpi: true,
     ..Default::default()
    }, |ctx| {
        unsafe { MINI_CONTEXT = Some(ctx) };

        UserData::free(Stage::new())
    });
}
