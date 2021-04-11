use miniquad::*;
use nvg_miniquad::nvgimpl;
// use nona::widgets::{Widget, Button};
// use nvg_miniquad::nvgimpl_orig as nvgimpl;

struct Stage<'a> {
    renderer: Option<nvgimpl::Renderer<'a>>,
    nona: nona::Context<nvgimpl::Renderer<'a>>,
}

static mut MINI_CONTEXT: Option<Context> = None;
fn get_context() -> &'static mut Context {
    unsafe { MINI_CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

impl<'a> Stage<'a> {
    pub fn new() -> Stage<'a> {
        let mut renderer = nvgimpl::Renderer::create(get_context()).unwrap();
        let mut nona = nona::Context::create(&mut renderer).unwrap();
        nona.create_font_from_file("roboto", "examples/Roboto-Bold.ttf")
            .unwrap();
        Stage {
            renderer: Some(renderer),
            nona,
        }
    }
}

impl<'a> EventHandlerFree for Stage<'a> {
    fn update(&mut self) {}

    fn draw(&mut self) {
        // let ctx = get_context();

        let nona = &mut self.nona;
        nona.attach_renderer(self.renderer.take());

        nona.begin_frame().unwrap();

        nona.begin_path();
        // nona.rect((100.0, 100.0, 400.0, 300.0));
        nona.rounded_rect((100.0, 100.0, 400.0, 300.0), 30.0);
        nona.fill_paint(nona::Gradient::Linear {
            start: (100, 100).into(),
            end: (400, 400).into(),
            start_color: nona::Color::rgb_i(0xAA, 0x6C, 0x39),
            end_color: nona::Color::rgb_i(0x88, 0x2D, 0x60),
        });
        nona.fill().unwrap();

        nona.begin_path();
        nona.font("roboto");
        nona.font_size(40.0);
        nona.text_align(nona::Align::TOP | nona::Align::LEFT);
        nona.fill_paint(nona::Color::rgb(1.0, 1.0, 1.0));
        nona.text((10, 10), format!("alpha texture font - working!!!"))
            .unwrap();

        // nona.begin_path();
        // nona.rect((100.0, 100.0, 300.0, 300.0));
        // nona.fill_paint(nona::Gradient::Linear {
        //     start: (100, 100).into(),
        //     end: (400, 400).into(),
        //     start_color: nona::Color::rgb_i(0xAA, 0x6C, 0x39),
        //     end_color: nona::Color::rgb_i(0x88, 0x2D, 0x60),
        // });
        // nona.fill().unwrap();

        let origin = (150.0, 140.0);
        nona.begin_path();
        nona.circle(origin, 64.0);
        nona.move_to(origin);
        nona
            .line_to((origin.0 + 300.0, origin.1 - 50.0));
        nona
            .stroke_paint(nona::Color::rgba(1.0, 1.0, 0.0, 1.0));
        nona.stroke_width(3.0);
        nona.stroke().unwrap();

        nona.end_frame().unwrap();

        // nona.save();
        // nona.global_composite_operation(nona::CompositeOperation::Basic(nona::BasicCompositeOperation::Lighter));
        // let origin = (150.0, 140.0);
        // nona.begin_path();
        // nona.circle(origin, 64.0);
        // nona.move_to(origin);
        // nona.line_join(nona::LineJoin::Round);
        // nona.line_to((origin.0 + 300.0, origin.1 - 50.0));
        // nona.quad_to((300.0, 100.0), (origin.0 + 500.0, origin.1 + 100.0));
        // nona.close_path();
        // nona.fill_paint(nona::Color::rgba(0.2, 0.0, 0.8, 1.0));
        // nona.fill().unwrap();
        // nona.stroke_paint(nona::Color::rgba(1.0, 1.0, 0.0, 1.0));
        // nona.stroke_width(3.0);
        // nona.stroke().unwrap();
        // nona.restore();

        // experimental, not yet done
        // let btn = Button {
        //     widget: Widget {
        //         width: 120.0,
        //         height: 40.0,
        //         ..Default::default()
        //     }
        // };

        // btn.draw(nona).unwrap();

        nona.end_frame().unwrap();

        self.renderer = nona.detach_renderer();
    }
}

fn main() {
    // color_backtrace::install();

    miniquad::start(
        conf::Conf {
            high_dpi: true,
            ..Default::default()
        },
        |ctx| {
            unsafe { MINI_CONTEXT = Some(ctx) };

            UserData::free(Stage::new())
        },
    );
}
