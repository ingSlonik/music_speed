use clap::{App, Arg};
use std::path::Path;

use music_speed::*;

fn main() {
    let matches = App::new("Music Speed")
        .version("0.1.0")
        .author("Filip Paulů <ing.fenix@seznam.cz>")
        .about("Analyze of tempo of music for each second.")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("file")
                .help("Sets the input file to use")
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
                .short("t")
                .long("time")
                .default_value("1000")
                .help("Interval time for each is music analyzed [ms]"),
        )
        .arg(
            Arg::with_name("analysis_interval")
                .short("a")
                .long("analysis")
                .default_value("3000")
                .help("Length of analyzing segment [ms]"),
        )
        .arg(
            Arg::with_name("min_bpm")
                .long("min")
                .default_value("80")
                .help("The minimal value sought [BPM]"),
        )
        .arg(
            Arg::with_name("max_bpm")
                .long("max")
                .default_value("160")
                .help("The maximal value sought  [BPM]"),
        )
        .get_matches();

    println!("Welcome in Music Speed v0.1.0.\n");

    let file_path = match matches.value_of("input") {
        None => panic!("The input file has to be set."),
        Some(file_path) => file_path,
    };

    let time_interval = match matches.value_of("time_interval").unwrap().parse::<usize>() {
        Ok(n) => n,
        Err(_) => panic!("The time_interval have to be integer."),
    };
    let analysis_interval = match matches
        .value_of("analysis_interval")
        .unwrap()
        .parse::<usize>()
    {
        Ok(n) => n,
        Err(_) => panic!("The analysis_interval have to be integer."),
    };
    let min_bpm = match matches.value_of("min_bpm").unwrap().parse::<usize>() {
        Ok(n) => n,
        Err(_) => panic!("The min_bpm have to be integer."),
    };
    let max_bpm = match matches.value_of("max_bpm").unwrap().parse::<usize>() {
        Ok(n) => n,
        Err(_) => panic!("The max_bpm have to be integer."),
    };

    let verbose = match matches.value_of("verbose").unwrap() {
        "0" => 0,
        "1" => 1,
        _ => panic!("Unknown verbose. allowed values are \"0\" and \"1\"."),
    };

    if !Path::new(file_path).exists() {
        panic!("File \"{}\" doesn't exist.", file_path);
    }

    let result = analyse_sync(Configuration {
        file_path,
        time_interval,
        analysis_interval,
        min_bpm,
        max_bpm,
        verbose,
    });

    println!("\nTime s\tBPM");
    result
        .into_iter()
        .for_each(|bpm| println!("{}\t{}", bpm.time, bpm.bpm));
}

// TODO: it wait for return...
/*
fn exit(message: &str) {
    println!("{}", message);
    std::process::exit(1);
}
*/
