use std::io::Write;
use std::fs::File;
use std::io::BufRead;
use regex::Regex;
use std::str::FromStr;

const SAMPLES_PER_SECOND: u128 = 16000;
const CHANNEL_COUNT: u8 = 1;

fn scale_time_exp (time_in: u128, frequency_start: f64, frequency_stop: f64, duration: f64) -> f64 { // Returns a virtual time in seconds
    let ln_ratio = (frequency_stop / frequency_start).ln();
    
    (frequency_start * duration / ln_ratio) * (( ln_ratio * ( (time_in as f64) / ( (SAMPLES_PER_SECOND as f64) * duration ))).exp() - 1.0)
}

fn scale_time_linear (time_in: u128, frequency: f64) -> f64 {
    frequency * (time_in as f64) / (SAMPLES_PER_SECOND as f64)
}

fn scale_time_sin (time_in: u128, frequency_a: f64, frequency_b: f64, meta_frequency: f64) -> f64 {
    let frequency_diff = ( frequency_a - frequency_b ) / 2.0; // Hz
    let frequency_mid = frequency_b + frequency_diff; // Hz
    let time = (time_in as f64) / (SAMPLES_PER_SECOND as f64); // Seconds
    
    frequency_mid * time + frequency_diff * (1.0/(std::f64::consts::TAU*meta_frequency)*(meta_frequency*std::f64::consts::TAU*time).sin())
}

fn write_sample (amplitude: f64) -> () {
    let mut a: f64 = amplitude * 255.0;
    if a > 255.0 { a = 255.0; }
    if a < 0.0   { a = 0.0;   }
    let _ = std::io::stdout().write(&[ a as u8 ]);
}

#[derive(Copy, Clone)]
enum WaveForm {
    Square,
    Triangle,
    Sine,
    Pulse(f64),
    SawTooth
}

#[derive(Debug)]
struct ParseError;
impl FromStr for WaveForm {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
	let sep = Regex::new(r"[(,)]+").expect("Invalid Regex");
	match sep.split(s).into_iter().filter(|e| e.len() > 0).collect::<Vec<&str>>() {
	    parts => {
		if parts.len() < 1 { return Err(ParseError); }
		match parts[0] {
		    "squ" => { Ok(WaveForm::Square) },
		    "tri" => { Ok(WaveForm::Triangle) },
		    "sin" => { Ok(WaveForm::Sine) },
		    "saw" => { Ok(WaveForm::SawTooth) },
		    "pul" => {
			if parts.len() < 2 { return Err(ParseError); }
			match parts[1].parse() {
			    Ok(v) => {Ok(WaveForm::Pulse(v))},
			    Err(_) => {Err(ParseError)}
			}
		    },
		    _ => { Err(ParseError) }
		}
	    },
	}
    }
}

impl WaveForm {
    fn audio_at (self, virt_time: f64) -> f64 {
	match self {
	    WaveForm::Square => {
		if virt_time % 1.0 < 0.5 {
		    return 0.0;
		} else {
		    return 1.0;
		}
	    },
	    WaveForm::Triangle => {
		if virt_time % 1.0 < 0.5 {
		    virt_time * 2.0
		} else {
		    (1.0-virt_time) * 2.0
		}
	    },
	    WaveForm::Sine => {
		(1.0+( virt_time * std::f64::consts::TAU ).sin())*0.5
	    },
	    WaveForm::Pulse(ratio) => {
		if virt_time % 1.0 < 1.0 - ratio {
		    0.0
		} else {
		    1.0
		}
	    },
	    WaveForm::SawTooth => {
		virt_time % 1.0
	    },
	}
    }
}

#[derive(Copy, Clone)]
struct Note {
    wave_form: WaveForm,
    volume: f64, // From 0 to 1
    frequency: f64, // In Hz
    duration: f64, // In beats
    time: f64, // In beats since last note
}

impl Note {
    fn new () -> Self {
	Self{wave_form: WaveForm::Square, volume: 1.0, frequency: 440.0, duration: 0.25, time: 0.0}
    }
    fn audio_at (self, time: f64) -> f64 {
	self.wave_form.audio_at(time * self.frequency) * self.volume
    }
}

#[derive(Copy, Clone)]
struct MetaData {
    tempo: f64,
    length: f64,
}

impl MetaData {
    fn new () -> Self {
	Self{tempo: 100.0, length: 16.0}
    }
}

fn print_wave( file: File ) -> () {
    let _ = std::io::stdout().write(&[82, 73, 70, 70, 36, 0, 0, 128, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, CHANNEL_COUNT, 0, 128, 62, 0, 0, 0, 0, 0, 0, 1, 0, 8, 0, 100, 97, 116, 97]);
    
    let lines = std::io::BufReader::new(file).lines();

    let mut default: Note = Note::new();
    let mut meta_data: MetaData = MetaData::new();
    let mut notes: Vec<Note> = Vec::<Note>::new();
    
    let sep  = Regex::new(r"[ \t]+").expect("Invalid Regex");
    let sep1 = Regex::new(r"[=]").expect("Invalid Regex");
    for l in lines {
	match l {
	    Result::Ok(line) => {
		let pieces: Vec<String> = sep.split(line.as_str()).into_iter().map(|e| e.to_string()).collect();
		match pieces[0].as_str() {
		    "META" => {
			eprint!("Meta Data\n");
			for piece in &pieces[1..] {
			    let halves: Vec<String> = sep1.split(piece.as_str()).into_iter().map(|e| e.to_string()).collect();
			    eprint!("\t{}\t{}\n", halves[0], halves[1]);
			    match halves[0].as_str() {
				"tempo" => { meta_data.tempo = halves[1].parse().unwrap(); },
				"length" => { meta_data.length = halves[1].parse().unwrap(); },
				huh => { eprint!("Unrecognised option: {}\n", huh); }
			    }
			}
		    },
		    "DEFAULT" => {
			eprint!("Note Default\n");
			for piece in &pieces[1..] {
			    let halves: Vec<String> = sep1.split(piece.as_str()).into_iter().map(|e| e.to_string()).collect();
			    eprint!("\t{}\t{}\n", halves[0], halves[1]);
			    match halves[0].as_str() {
				"wave" => { default.wave_form = halves[1].parse().unwrap(); },
				"volume" => { default.volume = halves[1].parse().unwrap(); },
				"frequency" => { default.frequency = halves[1].parse().unwrap(); },
				"pitch" => { default.frequency = halves[1].parse().unwrap(); }, // Placeholder
				"duration" => { default.duration = halves[1].parse().unwrap(); },
				"time" => { default.time = halves[1].parse().unwrap(); },
				huh => { eprint!("Unrecognised option: {}\n", huh); }
			    }
			}
		    },
		    "NOTE" => {
			eprint!("Note\n");
			let mut note: Note = default.clone();
			for piece in &pieces[1..] {
			    let halves: Vec<String> = sep1.split(piece.as_str()).into_iter().map(|e| e.to_string()).collect();
			    eprint!("\t{}\t{}\n", halves[0], halves[1]);
			    match halves[0].as_str() {
				"wave" => { note.wave_form = halves[1].parse().unwrap(); },
				"volume" => { note.volume = halves[1].parse().unwrap(); },
				"frequency" => { note.frequency = halves[1].parse().unwrap(); },
				"pitch" => { note.frequency = halves[1].parse().unwrap(); }, // Placeholder
				"duration" => { note.duration = halves[1].parse().unwrap(); },
				"time" => { note.time = halves[1].parse().unwrap(); },
				huh => { eprint!("Unrecognised option: {}\n", huh); }
			    }
			}
			notes.push(note);
		    },
		    first_piece => {
			eprint!("Unrecognised command: {}\n", first_piece);
		    }
		}
	    },
	    Result::Err(err) => { eprint!("{:?}", err); }
	}
    }

    for sample_index in 0..(meta_data.length*(SAMPLES_PER_SECOND as f64)) as u128 {
	let current_time: f64 = ((sample_index as f64) * meta_data.tempo) / ((SAMPLES_PER_SECOND as f64) * 60.0); // In Beats
	let mut note_start_time_accumulator: f64 = 0.0;
	let mut audio_accumulator: f64 = 0.0;
	for note in &notes {
	    note_start_time_accumulator += note.time;
	    if current_time < note_start_time_accumulator { break; }
	    if current_time > note_start_time_accumulator + note.duration { continue; }
	    audio_accumulator += note.audio_at(current_time);
	}
	write_sample( audio_accumulator );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
	println!( "Please provide a file to make a .wav file from." );
	return;
    }
    match File::open(args[1].clone()) {
	Result::Ok(file) => { print_wave( file ); },
	Result::Err(err) => { eprint!("Error while opening file: {}", err); }
    }
}
