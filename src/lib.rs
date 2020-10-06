use std::fs::File;
use std::sync::mpsc::channel;
use std::thread;

use minimp3::{Decoder, Error, Frame};
use pbr::ProgressBar;
use rayon::prelude::*;

pub struct Configuration<'a> {
    pub file_path: &'a str,
    pub time_interval: usize, // measure each x [ms]
    pub analysis_interval: usize,
    pub min_bpm: usize,
    pub max_bpm: usize,
    pub verbose: usize,
}

pub struct BPM {
    pub time: f32,
    pub bpm: f32,
}

enum State {
    Start,
    Step,
    End,
}

fn get_mp3_decoder(file_path: &str) -> Decoder<File> {
    Decoder::new(File::open(file_path).unwrap())
}

fn get_mono(data: Vec<i16>, channels: usize) -> Vec<f32> {
    if channels == 1 {
        data.into_iter().map(|s| s as f32).collect()
    } else {
        let channels_f32 = channels as f32;
        let mut mono: Vec<f32> = Vec::with_capacity(data.len() / channels);
        let mut i: i16 = 0;
        let mut channel: f32 = 0.0;

        for sample in data {
            channel += sample as f32;
            i += 1;

            if i % channels as i16 == 0 {
                mono.push(channel / channels_f32);
                i = 0;
                channel = 0.0;
            }
        }

        mono
    }
}

fn get_music(mut decoder: Decoder<File>) -> (Vec<f32>, usize) {
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

    (music, rate as usize)
}

fn get_windows<'a>(
    music: &'a [f32],
    conf: &Configuration,
    sample_rate: usize,
) -> Vec<(usize, &'a [f32], &'a [f32])> {
    let window_size = sample_rate * conf.analysis_interval / 1000;
    let interval_size = sample_rate * conf.time_interval / 1000;

    let samples_from = conf.min_bpm * sample_rate / 60;
    let samples_to = conf.max_bpm * sample_rate / 60;

    let mut windows: Vec<(usize, &'a [f32], &'a [f32])> = Vec::new();

    for window_index in 0..((music.len() - window_size - samples_to) / interval_size) {
        let start_index = window_index * interval_size;
        let end_index = start_index + window_size;

        windows.push((
            window_index,
            &music[start_index..end_index],
            &music[(start_index + samples_from)..(end_index + samples_to)],
        ));
    }

    windows
}

fn get_correlation(win_a: &[f32], win_b: &[f32]) -> Vec<f32> {
    let size = win_b.len() - win_a.len();

    let mut correlation: Vec<f32> = Vec::with_capacity(size);

    for size_index in 0..size {
        let mut value = 0f32;
        for (win_a_index, win_a_sample) in win_a.iter().enumerate() {
            value += *win_a_sample * win_b[size_index + win_a_index];
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

pub fn analyse(conf: Configuration) -> Vec<BPM> {
    if conf.verbose > 0 {
        println!("Loading music...");
    }

    let decoder = get_mp3_decoder(conf.file_path);
    let (music, sample_rate) = get_music(decoder);

    if conf.verbose > 0 {
        println!(
            "File path: '{}'\nDuration: {}s\nSample rate: {}\nSamples: {}\n",
            conf.file_path,
            music.len() / sample_rate,
            sample_rate,
            music.len()
        );

        println!("Parsing music...");
    }

    let music_windows = get_windows(&music, &conf, sample_rate);

    if conf.verbose > 0 {
        println!("Parsed to {} windows\n", music_windows.len());
    }

    let samples_from = conf.min_bpm * sample_rate / 60;

    if conf.verbose > 0 {
        println!("Analyzing music...");
    }

    let (sender, receiver) = channel();

    if conf.verbose > 0 {
        let music_windows_len = music_windows.len() as u64;
        thread::spawn(move || {
            let mut pb = ProgressBar::new(music_windows_len);

            loop {
                match receiver.recv() {
                    Ok(state) => match state {
                        State::Start => {
                            pb.set(0);
                        }
                        State::Step => {
                            pb.inc();
                        }
                        State::End => {
                            pb.finish();
                            break;
                        }
                    },
                    Err(_) => {
                        pb.finish();
                        break;
                    }
                }
            }
        });
    }

    sender.send(State::Start).unwrap();
    let end_sender = sender.clone();

    let bpms: Vec<f32> = music_windows
        .into_par_iter()
        .map_with(sender, |s, windows| {
            let correlation = get_correlation(windows.1, windows.2);

            let (_, index) = get_max(correlation);
            let bpm = (samples_from + index) as f32 * 60.0 / sample_rate as f32;
            s.send(State::Step).unwrap();

            bpm
        })
        .collect();

    end_sender.send(State::End).unwrap();

    let result = bpms
        .iter()
        .enumerate()
        .map(|(index, bpm)| BPM {
            time: (index * conf.time_interval) as f32 / 1000f32,
            bpm: *bpm,
        })
        .collect();

    /*
    if conf.verbose > 0 {
        result
            .into_iter()
            .for_each(|bpm| println!("{}s BPM: {}", bpm.time, bpm.bpm));
    }
    */

    result
}
