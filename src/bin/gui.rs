use std::sync::mpsc::channel;
use std::thread;

use druid::im::{vector, Vector};
use druid::widget::{Button, Flex, Label};
use druid::{ commands, Command, Handled, AppDelegate, DelegateCtx, Target, AppLauncher, Env, Data, Lens, LocalizedString, Widget, FontDescriptor, FontFamily, WindowDesc };

use music_speed::*;

#[derive(Clone, Data)]
struct BPMState {
    time: f32,
    bpm: f32,
}

#[derive(Clone, Data, Lens)]
struct AppState {
    file_path: String,
    result: Vector<BPMState>,
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
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            
            data.file_path = String::from(file_info.path().to_str().unwrap());

            return Handled::Yes;
        }
        Handled::No
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

        let save_dialog_options = 
    let open_dialog_options =FileDialogOptions::new()
    .allowed_types(vec![rs, txt, other])
    .default_type(txt)
    .default_name(default_save_name)
    .name_label("Target")
    .title("Choose a target for this lovely file")
    .button_text("Export");
        .default_name("MySavedFile.txt")
        .name_label("Source")
        .title("Where did you put that file?")
        .button_text("Import");

    let select_file_button = Button::new("Select .mp3 file")
        .on_click(|ctx, data: &mut AppState, _env| {
            match open_dialog() {
                Some(file_path) => data.file_path = file_path,
                _ => println!("No file")
            };
        });

    let selected_file = Label::new(|data: &AppState, _env: &Env| {
        if data.file_path.len() == 0 {
            "mp3 is not selected".to_string()
        } else {
            data.file_path.to_string()
        }
    });

    let analyse_button = Button::new("Analyse").on_click(|_ctx, data: &mut AppState, _env| {
        run_analyse(data, &data.file_path);
    });

    Flex::column()
        .with_child(h1)
        .with_child(select_file_button)
        .with_child(selected_file)
        .with_child(analyse_button)
}


fn open_dialog() -> Option<String> {
    let result = nfd::open_file_dialog(Some("mp3"), None).unwrap_or_else(|e| {
        panic!(e);
    });

    match result {
        Response::Okay(file_path) => Some(file_path),
        // Response::Cancel => String::from(""),
        _ => None,
    }
}

fn run_analyse(data: &mut AppState, file_path: &str) {
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
                    // handle
                    //     .dispatch(move |webview| webview.eval(&format!("start({})", size)))
                    //     .unwrap();
                }
                State::Step(bpm) => {
                    data.result.push_back(bpm);
                    //handle
                    //    .dispatch(move |webview| {
                    //        webview.eval(&format!("step({}, {})", bpm.time, bpm.bpm))
                    //    })
                    //    .unwrap();
                }
                State::End => {
                    //handle
                    //    .dispatch(move |webview| webview.eval(&format!("end()")))
                    //    .unwrap();
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
