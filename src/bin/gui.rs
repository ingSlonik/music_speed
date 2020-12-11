use std::sync::mpsc::channel;
use std::thread;

use druid::im::{vector, Vector};
use druid::kurbo::{Circle, Line};
use druid::lens::{self, LensExt};
use druid::widget::{Button, Either, Flex, Label, List, Painter, ProgressBar};
use druid::RenderContext;
use druid::{
    commands, AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, ExtEventSink,
    FileDialogOptions, FileSpec, FontDescriptor, FontFamily, Handled, Lens, LocalizedString,
    PaintCtx, Selector, Target, Widget, WidgetExt, WindowDesc,
};

use music_speed::*;

const START: Selector<usize> = Selector::new("start");
const STEP: Selector<BPM> = Selector::new("step");
const END: Selector<()> = Selector::new("end");

#[derive(Clone, Data, Lens)]
struct BPMState {
    time: f32,
    bpm: f32,
}

#[derive(Clone, Data, Lens)]
struct AppState {
    file_path: String,
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
            println!("Time: {}, BPM: {}", bpm.time, bpm.bpm);
            data.result.push_front(BPMState {
                time: bpm.time,
                bpm: bpm.bpm,
            });
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
        .window_size((600.0, 400.0));

    let data = AppState {
        file_path: "".into(),
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

    let select_file_button =
        Button::new("Select .mp3 file").on_click(move |ctx, data: &mut AppState, _env| {
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
            run_analyse(ctx.get_external_handle(), &data.file_path);
        }),
        Label::new("You have to select file").padding(5.0),
    );

    let progress = Either::<AppState>::new(
        |data, _env| data.is_analyzing,
        ProgressBar::new().lens(AppState::progress),
        Label::new("You have to run analyse").padding(5.0),
    );

    let result = Painter::<AppState>::new(|ctx, data, _| {
        chart(ctx, &data.result);
    });

    Flex::column()
        .with_child(h1)
        .with_child(select_file_button)
        .with_child(selected_file)
        .with_child(analyse_button)
        .with_child(progress)
        .with_flex_child(result, 1.0)
}

// TODO: temporary
fn chart(ctx: &mut PaintCtx, data: &Vector<BPMState>) {
    let bounds = ctx.size().to_rect();
    let dot_diam = bounds.width().max(bounds.height()) / 20.;
    let dot_spacing = dot_diam * 1.8;
    for y in 0..((bounds.height() / dot_diam).ceil() as usize) {
        for x in 0..((bounds.width() / dot_diam).ceil() as usize) {
            let x_offset = (y % 2) as f64 * (dot_spacing / 2.0);
            let x = x as f64 * dot_spacing + x_offset;
            let y = y as f64 * dot_spacing;
            let circ = Circle::new((x, y), dot_diam / 2.0);
            let purp = Color::rgb(1.0, 0.22, 0.76);
            ctx.fill(circ, &purp);
        }
    }
}

fn run_analyse(sink: ExtEventSink, file_path: &str) {
    let (sender, receiver) = channel();

    analyse(
        sender,
        Configuration {
            file_path: &file_path,
            time_interval: 1000,
            analysis_interval: 2000,
            min_bpm: 80,
            max_bpm: 160,
            verbose: 1,
        },
    );

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
                //handle
                //    .dispatch(move |webview| webview.eval(&format!("log('Error!')")))
                //    .unwrap();
                break;
            }
        }
    });
}
