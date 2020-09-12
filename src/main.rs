use minimp3::{Decoder, Error, Frame};

use std::fs::File;

fn get_configuration<'a>() -> (&'a str, i32, i32, i32) {
    let file_path = ".\\example\\mix.mp3";
    let min_bpm: i32 = 90;
    let time_interval: i32 = 1000; // measure each x [ms]
                                   // let analysis_interval: i32 = 12000; // good results
    let analysis_interval: i32 = 5000;
    (file_path, min_bpm, time_interval, analysis_interval)
}

fn get_mp3_decoder(file_path: &str) -> Decoder<File> {
    Decoder::new(File::open(file_path).unwrap())
}

fn get_mono(data: Vec<i16>, channels: usize) -> Vec<i16> {
    if channels == 1 {
        data
    } else {
        let mut mono: Vec<i16> = Vec::with_capacity(data.len() / channels);
        let mut i: i16 = 0;
        let mut channel: i32 = 0;

        for sample in data {
            channel += sample as i32;
            i = i + 1;

            if i % channels as i16 == 0 {
                mono.push((channel / channels as i32) as i16);
                i = 0;
                channel = 0;
            }
        }

        mono
    }
}

fn get_music(mut decoder: Decoder<File>) -> (Vec<i16>, i32) {
    let mut music = Vec::new();
    let mut rate = 0;

    loop {
        match decoder.next_frame() {
            Ok(Frame {
                data,
                channels,
                sample_rate,
                ..
            }) => {
                let mono = get_mono(data, channels);
                for sample in mono {
                    music.push(sample);
                }
                rate = sample_rate;
            }
            Err(Error::Eof) => break,
            Err(e) => panic!("{:?}", e),
        }
    }

    (music, rate)
}

fn get_windows(
    music: &Vec<i16>,
    sample_rate: i32,
    time_interval: i32,
    analysis_interval: i32,
) -> Vec<Vec<i16>> {
    let window_size = (sample_rate * analysis_interval / 1000) as usize;
    let time_interval_size = (sample_rate * time_interval / 1000) as usize;

    let mut windows: Vec<Vec<i16>> = Vec::new();

    for window_index in 0..((music.len() - window_size) / time_interval_size) {
        let mut window: Vec<i16> = Vec::with_capacity(window_size);
        let start_index = window_index * time_interval_size;
        let end_index = start_index + window_size;
        for i in start_index..end_index {
            window.push(music[i]);
        }
        windows.push(window);
    }

    windows
}

fn get_correlation(
    music: &Vec<i16>,
    window: &Vec<i16>,
    correlation_from: i32,
    correlation_to: i32,
) -> Vec<f32> {
    let window_size = window.len();

    let mut correlation: Vec<f32> =
        Vec::with_capacity((correlation_to - correlation_from + 1) as usize);
    let mut value;

    for music_index in correlation_from as usize..correlation_to as usize {
        value = 0.0;
        for window_index in 0..window_size - 1 {
            value += window[window_index] as f32 * music[music_index + window_index] as f32;
        }
        correlation.push(value);
    }

    correlation
}

fn get_max(data: Vec<f32>) -> (f32, usize) {
    let mut value = std::f32::MIN;
    let mut index = 0;

    for (i, x) in data.into_iter().enumerate() {
        if x > value {
            value = x;
            index = i;
        }
    }

    (value, index)
}

fn get_bpm(
    music: Vec<i16>,
    music_windows: Vec<Vec<i16>>,
    sample_rate: i32,
    time_interval: i32,
    min_bpm: i32,
    max_bpm: i32,
) -> Vec<f32> {
    let music_windows_len = music_windows.len();
    let time_interval_size = sample_rate * time_interval / 1000;
    let samples_from = min_bpm * sample_rate / 60;
    let samples_to = max_bpm * sample_rate / 60;

    let mut bpms: Vec<f32> = Vec::with_capacity(music_windows_len);

    for (i, window) in music_windows.into_iter().enumerate() {
        let correlation_from = i as i32 * time_interval_size + samples_from;
        let correlation_to = i as i32 * time_interval_size + samples_to;

        let correlation = get_correlation(&music, &window, correlation_from, correlation_to);
        // TODO: next version filter the corelation result?
        let (_, index) = get_max(correlation);
        let bpm = (samples_from + index as i32) as f32 * 60.0 / sample_rate as f32;
        bpms.push(bpm);

        println!(
            "{}s: {} BPM ({}%) - {}",
            i as i32 * time_interval / 1000,
            bpm,
            100 * i / music_windows_len,
            index,
        );
    }

    bpms
}

/*
fn show_results(results: Vec<i32>, time_interval: i32) {
    let mut time = 0;
    for result in results {
        println!("{}s: {} BPM", time / 1000, result);
        time = time + time_interval;
    }
}

fn get_avg(data: &Vec<i16>) -> i16 {
    let len = data.len() as i32;
    let mut sum: i32 = 0;
    for sample in data {
        sum += *sample as i32;
    }

    (sum / len) as i16
}
*/

fn main() {
    println!("Welcome in Speed of Music v0.1.0.\n");

    let (file_path, min_bpm, time_interval, analysis_interval) = get_configuration();
    let max_bpm = min_bpm * 2;

    println!("Loading music...");
    let decoder = get_mp3_decoder(file_path);
    let (music, sample_rate) = get_music(decoder);
    println!(
        "File path: '{}'\nDuration: {}s\nSample rate: {}\nSamples: {}\n",
        file_path,
        music.len() as i32 / sample_rate,
        sample_rate,
        music.len()
    );

    println!("Parsing music...");
    let music_windows = get_windows(&music, sample_rate, time_interval, analysis_interval);
    println!("Parsed to {} windows\n", music_windows.len());

    println!("Analysing music...");
    get_bpm(
        music,
        music_windows,
        sample_rate,
        time_interval,
        min_bpm,
        max_bpm,
    );

    println!("Done :)");
    // Show results in analysing
    // show_results(results, time_interval);
}
