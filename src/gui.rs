use music_speed::*;
use nfd::Response;
use std::include_str;
use std::sync::mpsc::channel;
use std::thread;
use web_view::{Content, WebView};

pub fn run() {
    let html = include_str!("gui/dist/index.html");
    let css = include_str!("gui/dist/style.css");
    let js = include_str!("gui/dist/bundle.js");

    let build_html = html
        .replace(
            "<link rel=\"stylesheet\" href=\"style.css\" />",
            &format!("<style>{}</style>", css),
        )
        .replace(
            "<script src=\"bundle.js\"></script>",
            &format!("<script>{}</script>", js),
        );

    web_view::builder()
        .title("Music Speed")
        .content(Content::Html(build_html))
        .size(640, 480)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| match arg {
            "open_dialog" => {
                match open_dialog() {
                    None => webview.eval("log('No file')").unwrap(),
                    Some(file_path) => {
                        run_analyse(webview, &file_path);
                    }
                };
                Ok(())
            }
            _ => unimplemented!(),
        })
        .run()
        .unwrap();
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

fn run_analyse(webview: &mut WebView<()>, file_path: &str) {
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

    let handle = webview.handle();

    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(state) => match state {
                State::Start(size) => {
                    handle
                        .dispatch(move |webview| webview.eval(&format!("start({})", size)))
                        .unwrap();
                }
                State::Step(bpm) => {
                    handle
                        .dispatch(move |webview| {
                            webview.eval(&format!("step({}, {})", bpm.time, bpm.bpm))
                        })
                        .unwrap();
                }
                State::End => {
                    handle
                        .dispatch(move |webview| webview.eval(&format!("end()")))
                        .unwrap();
                    break;
                }
            },
            Err(_) => {
                handle
                    .dispatch(move |webview| webview.eval(&format!("log('Error!')")))
                    .unwrap();
                break;
            }
        }
    });
}
