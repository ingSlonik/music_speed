# Music speed

Analyze of tempo of music for each second.

## Motivation

## Library

```rust
use music_speed::*;

fn main() {
    let result = analyse(Configuration {
        file_path: "./path/to.mp3",
        time_interval: 1000, // [ms]
        analysis_interval: 2000, // [ms]
        min_bpm: 90,
        max_bpm: 180,
        verbose: 1,
    });
}
```

## CLI

```
$ music_speed --input ./path/to.mp3
```

## GUI

### Develop

```
$ npm run tauri dev
```
