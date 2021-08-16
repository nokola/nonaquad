use miniquad::*;
use nona::{Align, Color, Gradient, Paint, Point};
use nonaquad::nvgimpl;
// use nona::widgets::{Widget, Button};
// use nonaquad::nvgimpl_orig as nvgimpl;

struct Stage {
    renderer: nvgimpl::Renderer,
    nona: nona::Context,
}

impl Stage {
    pub fn new(ctx: &mut Context) -> Stage {
        let mut renderer = nvgimpl::Renderer::create(ctx).unwrap();
        let mut nona =
            nona::Context::create(&mut nvgimpl::RendererCtx::new(&mut renderer, ctx)).unwrap();

        // for demo: load font by embedding into binary
        let font_data: &'static [u8] = include_bytes!("Roboto-Bold.ttf");
        nona.create_font("roboto", font_data).unwrap();

        // use this to load fonts dynamically at runtime:
        // nona.create_font_from_file("roboto", "examples/Roboto-Bold.ttf")
        //     .unwrap();
        Stage { renderer, nona }
    }
}

impl EventHandler for Stage {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        // let ctx = get_context();

        self.nona.attach_renderer(
            &mut nvgimpl::RendererCtx::new(&mut self.renderer, ctx),
            |canvas| {
                canvas
                    .begin_frame(Some(Color::rgb_i(128, 128, 255)))
                    .unwrap();

                // uncomment to draw a lot of circles - more than maximum GPU vertices on openGL ES 2/WebGL
                // note: performance is currently low, very CPU-bound. Something to fix in the future.
                // for i in 0..405 {
                //     canvas.begin_path();
                //     // canvas.rect((100.0, 100.0, 400.0, 300.0));
                //     canvas.circle(Point::new(i as f32, 110.), 32.);
                //     canvas.fill_paint(Paint::from(Color::rgb_i(255, (i as u32 % 256 as u32) as u8, 0)));
                //     canvas.fill().unwrap();
                // }

                canvas.begin_path();
                // canvas.rect((100.0, 100.0, 400.0, 300.0));
                canvas.rounded_rect((100.0, 100.0, 400.0, 300.0), 30.0);
                canvas.fill_paint(Gradient::Linear {
                    start: (100, 100).into(),
                    end: (400, 400).into(),
                    start_color: Color::rgb_i(0xAA, 0x6C, 0x39),
                    end_color: Color::rgb_i(0x88, 0x2D, 0x60),
                });
                canvas.fill().unwrap();

                canvas.begin_path();
                canvas.font("roboto");
                canvas.font_size(40.0);
                canvas.text_align(Align::TOP | Align::LEFT);
                canvas.fill_paint(Color::rgb(1.0, 1.0, 1.0));
                canvas
                    .text((10, 10), format!("alpha texture font - working!!!"))
                    .unwrap();

                // canvas.begin_path();
                // canvas.rect((100.0, 100.0, 300.0, 300.0));
                // canvas.fill_paint(nona::Gradient::Linear {
                //     start: (100, 100).into(),
                //     end: (400, 400).into(),
                //     start_color: nona::Color::rgb_i(0xAA, 0x6C, 0x39),
                //     end_color: nona::Color::rgb_i(0x88, 0x2D, 0x60),
                // });
                // canvas.fill().unwrap();

                let origin = (150.0, 140.0);
                canvas.begin_path();
                canvas.circle(origin, 64.0);
                canvas.move_to(origin);
                canvas.line_to((origin.0 + 300.0, origin.1 - 50.0));
                canvas.stroke_paint(Color::rgba(1.0, 1.0, 0.0, 1.0));
                canvas.stroke_width(3.0);
                canvas.stroke().unwrap();

                canvas.end_frame().unwrap();
            },
        );

        ctx.commit_frame();
    }
}

fn main() {
    // color_backtrace::install();

    miniquad::start(
        conf::Conf {
            high_dpi: true,
            ..Default::default()
        },
        |mut ctx| UserData::owning(Stage::new(&mut ctx), ctx),
    );
}
