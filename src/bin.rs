use clap::{App, Arg};
use music_speed::*;

fn main() {
    let matches = App::new("Speed of Music")
        .version("0.1")
        .author("Filip Paulů <ing.fenix@seznam.cz>")
        .about("Analyze of tempo of music for each second.")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("file")
                .help("Sets the input file to use")
                .required(true)
                .index(1)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .default_value("1")
                .help("Sets the level of verbosity. 0 - None, 1 - Full"),
        )
        .arg(
            Arg::with_name("time_interval")
                .short("i")
                .long("interval")
                .default_value("1000")
                .help("Interval for each is music analyzed."),
        )
        .arg(
            Arg::with_name("analysis_interval")
                .short("a")
                .long("analysis")
                .default_value("3000")
                .help("Length of analyzing segment."),
        )
        .arg(
            Arg::with_name("min_bpm")
                .long("min")
                .help("The minimal value sought."),
        )
        .arg(
            Arg::with_name("max_bpm")
                .long("max")
                .help("The maximal value sought."),
        )
        .get_matches();

    println!("Welcome in Music Speed v0.1.0.\n");

    let result = analyse(Configuration {
        file_path: ".\\__example\\mix10s.mp3",
        time_interval: 1000,
        analysis_interval: 2000,
        min_bpm: 90,
        max_bpm: 180,
        verbose: 1,
    });

    println!("Time [s]\tBPM");
    result
        .into_iter()
        .for_each(|bpm| println!("{}\t{}", bpm.time, bpm.bpm));
}
