use clap::{App, Arg};
use speed_of_music::*;

fn main() {
    let matches = App::new("Speed of Music")
        .version("0.1")
        .author("Filip Paulů <ing.fenix@seznam.cz>")
        .about("Analyze of tempo of music for each second.")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input file to use")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Sets a output config file")
                .help("Sets the input file to use")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    // println!("Welcome in Speed of Music v0.1.0.\n");
    let result = analyse(Configuration {
        file_path: ".\\__example\\mix10s.mp3",
        time_interval: 500,
        analysis_interval: 2000,
        min_bpm: 90,
        max_bpm: 180,
        verbose: 1,
    });

    result
        .into_iter()
        .for_each(|bpm| println!("{}s BPM: {}", bpm.time, bpm.bpm));
}
