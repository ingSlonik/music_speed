use std::sync::mpsc::channel;
use std::thread;

use druid::im::{vector, Vector};
use druid::kurbo::{Circle, Line};
// use druid::lens::{self};
use druid::piet::{FontFamily, Text, TextAlignment, TextLayoutBuilder};
use druid::widget::{Button, Either, Flex, Label, Painter, ProgressBar, Slider};
use druid::RenderContext;
use druid::{
    commands, AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, ExtEventSink,
    FileDialogOptions, FileSpec, FontDescriptor, Handled, Lens, LocalizedString, PaintCtx,
    Selector, Target, Widget, WidgetExt, WindowDesc,
};

use music_speed::*;

const START: Selector<usize> = Selector::new("start");
const STEP: Selector<BPM> = Selector::new("step");
const END: Selector<()> = Selector::new("end");

#[derive(Clone, Data, Lens)]
struct BPMState {
    time: usize,
    bpm: f32,
}

#[derive(Clone, Data, Lens)]
struct AppState {
    file_path: String,
    time_interval: f64,
    analysis_interval: f64,
    min_bpm: f64,
    max_bpm: f64,
    result: Vector<BPMState>,
    is_analyzing: bool,
    size: usize,
    progress: f64,
}

struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_path) = cmd.get(commands::OPEN_FILE) {
            data.file_path = String::from(file_path.path().to_str().unwrap());
            Handled::Yes
        } else if let Some(size) = cmd.get(START) {
            println!("Size: {}", size);
            data.is_analyzing = true;
            data.size = *size;
            Handled::Yes
        } else if let Some(bpm) = cmd.get(STEP) {
            println!("\nTime: {:.2}, BPM: {}", bpm.time as f32 / 1000.0, bpm.bpm);
            data.result.push_front(BPMState {
                time: bpm.time,
                bpm: bpm.bpm,
            });
            data.result
                .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
            data.progress = data.result.len() as f64 / data.size as f64;
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

fn main() {
    println!("Welcome in Music Speed v0.1.0.\n");

    let main_window = WindowDesc::new(ui_builder)
        .title(LocalizedString::new("Music Speed"))
        .window_size((700.0, 600.0));

    let data = AppState {
        file_path: "".into(),
        time_interval: 2000f64,
        analysis_interval: 3500f64,
        min_bpm: 80f64,
        max_bpm: 160f64,
        result: vector![],
        is_analyzing: false,
        size: 0,
        progress: 0f64,
    };

    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<AppState> {
    let h1 = Label::new("Welcome in Music Speed v0.1.0")
        .with_font(FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(32.0));

    let open_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![FileSpec::new("Music", &["mp3"])])
        .name_label("Music")
        .title("Where did you put that file?")
        .button_text("Load");

    let select_file_button = Button::new("Select .mp3 file").on_click(move |ctx, _data, _env| {
        ctx.submit_command(Command::new(
            druid::commands::SHOW_OPEN_PANEL,
            open_dialog_options.clone(),
            Target::Auto,
        ))
    });

    let selected_file = Label::new(|data: &AppState, _env: &Env| {
        if data.file_path.len() == 0 {
            "mp3 is not selected".to_string()
        } else {
            data.file_path.to_string()
        }
    });

    let analyse_button = Either::<AppState>::new(
        |data, _env| data.file_path.len() > 0,
        Button::new("Analyse").on_click(move |ctx, data: &mut AppState, _env| {
            run_analyse(
                ctx.get_external_handle(),
                Configuration {
                    file_path: &data.file_path,
                    time_interval: data.time_interval as usize,
                    analysis_interval: data.analysis_interval as usize,
                    min_bpm: data.min_bpm as usize,
                    max_bpm: data.max_bpm as usize,
                    verbose: 1, // TODO: set to 0
                },
            );
        }),
        Label::new("You have to select file").padding(5.0),
    );

    let progress = Either::<AppState>::new(
        |data, _env| data.is_analyzing,
        Flex::row()
            .with_default_spacer()
            .with_flex_child(
                ProgressBar::new().expand_width().lens(AppState::progress),
                1.0,
            )
            .with_default_spacer()
            .with_child(Label::new(|data: &AppState, _env: &Env| {
                format!("{:.2} %", data.progress * 100.0)
            }))
            .with_default_spacer(),
        Label::new("You have to run analyse").padding(5.0),
    );

    let result = Painter::<AppState>::new(|ctx, data, _| {
        if data.result.len() > 0 {
            chart(ctx, &data);
        }
    });

    Flex::column()
        .with_child(h1)
        .with_default_spacer()
        .with_child(select_file_button)
        .with_default_spacer()
        .with_child(selected_file)
        .with_default_spacer()
        .with_child(
            Flex::row()
                .with_default_spacer()
                .with_flex_child(
                    Flex::column()
                        .with_child(number(
                            "Time interval:",
                            500f64,
                            5000f64,
                            AppState::time_interval,
                            |data| format!("{:.2} s", data.time_interval / 1000f64),
                        ))
                        .with_default_spacer()
                        .with_child(number(
                            "Analysis interval:",
                            1000f64,
                            10000f64,
                            AppState::analysis_interval,
                            |data| format!("{:.2} s", data.analysis_interval / 1000f64),
                        )),
                    1.0,
                )
                .with_default_spacer()
                .with_flex_child(
                    Flex::column()
                        .with_child(number("Min:", 20f64, 200f64, AppState::min_bpm, |data| {
                            format!("{:.0} BPM", data.min_bpm)
                        }))
                        .with_default_spacer()
                        .with_child(number("Max:", 40f64, 400f64, AppState::max_bpm, |data| {
                            format!("{:.0} BPM", data.max_bpm)
                        })),
                    1.0,
                )
                .with_default_spacer(),
        )
        .with_default_spacer()
        .with_child(analyse_button)
        .with_default_spacer()
        .with_child(progress)
        .with_default_spacer()
        .with_flex_child(result, 1.0)
}

fn number<S: Data, T: 'static + Lens<S, f64>>(
    label: &str,
    min: f64,
    max: f64,
    lens: T,
    text: fn(&S) -> String,
) -> Flex<S> {
    Flex::row()
        .with_child(Label::new(label))
        .with_flex_child(
            Slider::new().with_range(min, max).expand_width().lens(lens),
            1.0,
        )
        .with_child(Label::new(move |data: &S, _env: &Env| text(data)))
}

fn chart(ctx: &mut PaintCtx, state: &AppState) {
    let bounds = ctx.size().to_rect();

    let duration = state.size as f64 * state.time_interval;

    // axis
    let axis_width = 1.0;
    let axis_color = Color::rgb(1.0, 1.0, 1.0);
    let axis_label_color = Color::rgb(1.0, 1.0, 1.0);
    let axis_label_padding = 4.0;
    let axis_font_size = 14.0;
    let x_offset = 32.0;
    let y_offset = 24.0;

    let width = bounds.width() - x_offset * 2.0;
    let height = bounds.height() - y_offset * 2.0;

    // x axis
    {
        let line = Line::new(
            (x_offset, bounds.height() - y_offset),
            (bounds.width() - x_offset, bounds.height() - y_offset),
        );
        ctx.stroke(line, &axis_color, axis_width);

        // labels
        let min_text = ctx
            .text()
            .new_text_layout("0")
            .font(FontFamily::SERIF, axis_font_size)
            .text_color(axis_label_color.clone())
            .alignment(TextAlignment::Start)
            .build()
            .unwrap();
        ctx.draw_text(
            &min_text,
            (x_offset, height + y_offset + axis_label_padding),
        );

        let max_text = ctx
            .text()
            .new_text_layout(format!("{:.0} [s]", duration / 1000.0))
            .font(FontFamily::SERIF, axis_font_size)
            .text_color(axis_label_color.clone())
            .alignment(TextAlignment::End)
            .max_width(width)
            .build()
            .unwrap();
        ctx.draw_text(
            &max_text,
            (x_offset, height + y_offset + axis_label_padding),
        );
    }

    // y axis
    {
        let line = Line::new((x_offset, y_offset), (x_offset, bounds.height() - y_offset));
        ctx.stroke(line, &axis_color, axis_width);

        // labels
        let max_bpm_text = ctx
            .text()
            .new_text_layout(format!("{:.0}", state.max_bpm))
            .font(FontFamily::SERIF, axis_font_size)
            .text_color(axis_label_color.clone())
            .alignment(TextAlignment::End)
            .max_width(x_offset - axis_label_padding)
            .build()
            .unwrap();
        ctx.draw_text(&max_bpm_text, (0.0, y_offset - axis_font_size / 2.0));

        let min_bpm_text = ctx
            .text()
            .new_text_layout(format!("{:.0}", state.min_bpm))
            .font(FontFamily::SERIF, axis_font_size)
            .text_color(axis_label_color.clone())
            .alignment(TextAlignment::End)
            .max_width(x_offset - axis_label_padding)
            .build()
            .unwrap();
        ctx.draw_text(
            &min_bpm_text,
            (0.0, bounds.height() - y_offset - axis_font_size / 2.0),
        );
    }

    // Line
    let line_color = Color::rgb(0.0, 0.0, 1.0);
    let line_width = 2.0;
    let circle_radius = 4.0;

    {
        let mut x_before = 0.0;
        let mut y_before = 0.0;

        // the result is sorted by time
        for result in state.result.iter() {
            let x = result.time as f64 / duration * width;
            let y = ((result.bpm - state.min_bpm as f32)
                / (state.max_bpm as f32 - state.min_bpm as f32)) as f64
                * height;

            ctx.stroke(
                Line::new(
                    (x_offset + x_before, y_offset + y_before),
                    (x_offset + x, y_offset + y),
                ),
                &line_color,
                line_width,
            );
            ctx.fill(
                Circle::new((x_offset + x, y_offset + y), circle_radius),
                &line_color,
            );

            x_before = x;
            y_before = y;
        }
    }
}

fn run_analyse(sink: ExtEventSink, conf: Configuration) {
    let (sender, receiver) = channel();

    analyse(sender, conf);

    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(state) => match state {
                State::Start(size) => {
                    sink.submit_command(START, size, Target::Auto)
                        .expect("command failed to submit");
                }
                State::Step(bpm) => {
                    sink.submit_command(STEP, bpm, Target::Auto)
                        .expect("command failed to submit");
                }
                State::End => {
                    sink.submit_command(END, (), Target::Auto)
                        .expect("command failed to submit");
                    break;
                }
            },
            Err(_) => {
                // TODO: handle it
                break;
            }
        }
    });
}
