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

fn sample_data (amplitude: f64) -> [u8; 2] {
    let mut a: f64 = amplitude * 65535.0 - 32768.0;
    if a > 32767.0 { a = 32767.0; }
    if a < -32768.0   { a = -32768.0;   }
    let double_byte: i16 = a as i16;
    double_byte.to_le_bytes()
}

fn pitch_to_frequency (pitch_name: &str) -> Result<f64, ParseError> {
    match pitch_name {
         "A0" => { Ok(27.5) },
	"A#0" => { Ok(27.5 * 2.0_f64.powf(1.0/12.0)) },
        "Bb0" => { Ok(27.5 * 2.0_f64.powf(1.0/12.0)) },
	 "B0" => { Ok(27.5 * 2.0_f64.powf(2.0/12.0)) },
	 "C0" => { Ok(27.5 * 2.0_f64.powf(3.0/12.0)) },
	"C#0" => { Ok(27.5 * 2.0_f64.powf(4.0/12.0)) },
	"Db0" => { Ok(27.5 * 2.0_f64.powf(4.0/12.0)) },
	 "D0" => { Ok(27.5 * 2.0_f64.powf(5.0/12.0)) },
	"D#0" => { Ok(27.5 * 2.0_f64.powf(6.0/12.0)) },
	"Eb0" => { Ok(27.5 * 2.0_f64.powf(6.0/12.0)) },
	 "E0" => { Ok(27.5 * 2.0_f64.powf(7.0/12.0)) },
	 "F0" => { Ok(27.5 * 2.0_f64.powf(8.0/12.0)) },
	"F#0" => { Ok(27.5 * 2.0_f64.powf(9.0/12.0)) },
	"Gb0" => { Ok(27.5 * 2.0_f64.powf(9.0/12.0)) },
	 "G0" => { Ok(27.5 * 2.0_f64.powf(10.0/12.0)) },
	"G#0" => { Ok(27.5 * 2.0_f64.powf(11.0/12.0)) },
	"Ab0" => { Ok(27.5 * 2.0_f64.powf(11.0/12.0)) },
	 "A1" => { Ok(55.0) },
	"A#1" => { Ok(55.0 * 2.0_f64.powf(1.0/12.0)) },
        "Bb1" => { Ok(55.0 * 2.0_f64.powf(1.0/12.0)) },
	 "B1" => { Ok(55.0 * 2.0_f64.powf(2.0/12.0)) },
	 "C1" => { Ok(55.0 * 2.0_f64.powf(3.0/12.0)) },
	"C#1" => { Ok(55.0 * 2.0_f64.powf(4.0/12.0)) },
	"Db1" => { Ok(55.0 * 2.0_f64.powf(4.0/12.0)) },
	 "D1" => { Ok(55.0 * 2.0_f64.powf(5.0/12.0)) },
	"D#1" => { Ok(55.0 * 2.0_f64.powf(6.0/12.0)) },
	"Eb1" => { Ok(55.0 * 2.0_f64.powf(6.0/12.0)) },
	 "E1" => { Ok(55.0 * 2.0_f64.powf(7.0/12.0)) },
	 "F1" => { Ok(55.0 * 2.0_f64.powf(8.0/12.0)) },
	"F#1" => { Ok(55.0 * 2.0_f64.powf(9.0/12.0)) },
	"Gb1" => { Ok(55.0 * 2.0_f64.powf(9.0/12.0)) },
	 "G1" => { Ok(55.0 * 2.0_f64.powf(10.0/12.0)) },
	"G#1" => { Ok(55.0 * 2.0_f64.powf(11.0/12.0)) },
	"Ab1" => { Ok(55.0 * 2.0_f64.powf(11.0/12.0)) },
	 "A2" => { Ok(110.0) },
	"A#2" => { Ok(110.0 * 2.0_f64.powf(1.0/12.0)) },
        "Bb2" => { Ok(110.0 * 2.0_f64.powf(1.0/12.0)) },
	 "B2" => { Ok(110.0 * 2.0_f64.powf(2.0/12.0)) },
	 "C2" => { Ok(110.0 * 2.0_f64.powf(3.0/12.0)) },
	"C#2" => { Ok(110.0 * 2.0_f64.powf(4.0/12.0)) },
	"Db2" => { Ok(110.0 * 2.0_f64.powf(4.0/12.0)) },
	 "D2" => { Ok(110.0 * 2.0_f64.powf(5.0/12.0)) },
	"D#2" => { Ok(110.0 * 2.0_f64.powf(6.0/12.0)) },
	"Eb2" => { Ok(110.0 * 2.0_f64.powf(6.0/12.0)) },
	 "E2" => { Ok(110.0 * 2.0_f64.powf(7.0/12.0)) },
	 "F2" => { Ok(110.0 * 2.0_f64.powf(8.0/12.0)) },
	"F#2" => { Ok(110.0 * 2.0_f64.powf(9.0/12.0)) },
	"Gb2" => { Ok(110.0 * 2.0_f64.powf(9.0/12.0)) },
	 "G2" => { Ok(110.0 * 2.0_f64.powf(10.0/12.0)) },
	"G#2" => { Ok(110.0 * 2.0_f64.powf(11.0/12.0)) },
	"Ab2" => { Ok(110.0 * 2.0_f64.powf(11.0/12.0)) },
	 "A3" => { Ok(220.0) },
	"A#3" => { Ok(220.0 * 2.0_f64.powf(1.0/12.0)) },
        "Bb3" => { Ok(220.0 * 2.0_f64.powf(1.0/12.0)) },
	 "B3" => { Ok(220.0 * 2.0_f64.powf(2.0/12.0)) },
	 "C3" => { Ok(220.0 * 2.0_f64.powf(3.0/12.0)) },
	"C#3" => { Ok(220.0 * 2.0_f64.powf(4.0/12.0)) },
	"Db3" => { Ok(220.0 * 2.0_f64.powf(4.0/12.0)) },
	 "D3" => { Ok(220.0 * 2.0_f64.powf(5.0/12.0)) },
	"D#3" => { Ok(220.0 * 2.0_f64.powf(6.0/12.0)) },
	"Eb3" => { Ok(220.0 * 2.0_f64.powf(6.0/12.0)) },
	 "E3" => { Ok(220.0 * 2.0_f64.powf(7.0/12.0)) },
	 "F3" => { Ok(220.0 * 2.0_f64.powf(8.0/12.0)) },
	"F#3" => { Ok(220.0 * 2.0_f64.powf(9.0/12.0)) },
	"Gb3" => { Ok(220.0 * 2.0_f64.powf(9.0/12.0)) },
	 "G3" => { Ok(220.0 * 2.0_f64.powf(10.0/12.0)) },
	"G#3" => { Ok(220.0 * 2.0_f64.powf(11.0/12.0)) },
	"Ab3" => { Ok(220.0 * 2.0_f64.powf(11.0/12.0)) },
	 "A4" => { Ok(440.0) },
	"A#4" => { Ok(440.0 * 2.0_f64.powf(1.0/12.0)) },
        "Bb4" => { Ok(440.0 * 2.0_f64.powf(1.0/12.0)) },
	 "B4" => { Ok(440.0 * 2.0_f64.powf(2.0/12.0)) },
	 "C4" => { Ok(440.0 * 2.0_f64.powf(3.0/12.0)) },
	"C#4" => { Ok(440.0 * 2.0_f64.powf(4.0/12.0)) },
	"Db4" => { Ok(440.0 * 2.0_f64.powf(4.0/12.0)) },
	 "D4" => { Ok(440.0 * 2.0_f64.powf(5.0/12.0)) },
	"D#4" => { Ok(440.0 * 2.0_f64.powf(6.0/12.0)) },
	"Eb4" => { Ok(440.0 * 2.0_f64.powf(6.0/12.0)) },
	 "E4" => { Ok(440.0 * 2.0_f64.powf(7.0/12.0)) },
	 "F4" => { Ok(440.0 * 2.0_f64.powf(8.0/12.0)) },
	"F#4" => { Ok(440.0 * 2.0_f64.powf(9.0/12.0)) },
	"Gb4" => { Ok(440.0 * 2.0_f64.powf(9.0/12.0)) },
	 "G4" => { Ok(440.0 * 2.0_f64.powf(10.0/12.0)) },
	"G#4" => { Ok(440.0 * 2.0_f64.powf(11.0/12.0)) },
	"Ab4" => { Ok(440.0 * 2.0_f64.powf(11.0/12.0)) },
	 "A5" => { Ok(880.0) },
	"A#5" => { Ok(880.0 * 2.0_f64.powf(1.0/12.0)) },
        "Bb5" => { Ok(880.0 * 2.0_f64.powf(1.0/12.0)) },
	 "B5" => { Ok(880.0 * 2.0_f64.powf(2.0/12.0)) },
	 "C5" => { Ok(880.0 * 2.0_f64.powf(3.0/12.0)) },
	"C#5" => { Ok(880.0 * 2.0_f64.powf(4.0/12.0)) },
	"Db5" => { Ok(880.0 * 2.0_f64.powf(4.0/12.0)) },
	 "D5" => { Ok(880.0 * 2.0_f64.powf(5.0/12.0)) },
	"D#5" => { Ok(880.0 * 2.0_f64.powf(6.0/12.0)) },
	"Eb5" => { Ok(880.0 * 2.0_f64.powf(6.0/12.0)) },
	 "E5" => { Ok(880.0 * 2.0_f64.powf(7.0/12.0)) },
	 "F5" => { Ok(880.0 * 2.0_f64.powf(8.0/12.0)) },
	"F#5" => { Ok(880.0 * 2.0_f64.powf(9.0/12.0)) },
	"Gb5" => { Ok(880.0 * 2.0_f64.powf(9.0/12.0)) },
	 "G5" => { Ok(880.0 * 2.0_f64.powf(10.0/12.0)) },
	"G#5" => { Ok(880.0 * 2.0_f64.powf(11.0/12.0)) },
	"Ab5" => { Ok(880.0 * 2.0_f64.powf(11.0/12.0)) },
	 "A6" => { Ok(1760.0) },
	"A#6" => { Ok(1760.0 * 2.0_f64.powf(1.0/12.0)) },
        "Bb6" => { Ok(1760.0 * 2.0_f64.powf(1.0/12.0)) },
	 "B6" => { Ok(1760.0 * 2.0_f64.powf(2.0/12.0)) },
	 "C6" => { Ok(1760.0 * 2.0_f64.powf(3.0/12.0)) },
	"C#6" => { Ok(1760.0 * 2.0_f64.powf(4.0/12.0)) },
	"Db6" => { Ok(1760.0 * 2.0_f64.powf(4.0/12.0)) },
	 "D6" => { Ok(1760.0 * 2.0_f64.powf(5.0/12.0)) },
	"D#6" => { Ok(1760.0 * 2.0_f64.powf(6.0/12.0)) },
	"Eb6" => { Ok(1760.0 * 2.0_f64.powf(6.0/12.0)) },
	 "E6" => { Ok(1760.0 * 2.0_f64.powf(7.0/12.0)) },
	 "F6" => { Ok(1760.0 * 2.0_f64.powf(8.0/12.0)) },
	"F#6" => { Ok(1760.0 * 2.0_f64.powf(9.0/12.0)) },
	"Gb6" => { Ok(1760.0 * 2.0_f64.powf(9.0/12.0)) },
	 "G6" => { Ok(1760.0 * 2.0_f64.powf(10.0/12.0)) },
	"G#6" => { Ok(1760.0 * 2.0_f64.powf(11.0/12.0)) },
	"Ab6" => { Ok(1760.0 * 2.0_f64.powf(11.0/12.0)) },
	 "A7" => { Ok(3520.0) },
	"A#7" => { Ok(3520.0 * 2.0_f64.powf(1.0/12.0)) },
        "Bb7" => { Ok(3520.0 * 2.0_f64.powf(1.0/12.0)) },
	 "B7" => { Ok(3520.0 * 2.0_f64.powf(2.0/12.0)) },
	 "C7" => { Ok(3520.0 * 2.0_f64.powf(3.0/12.0)) },
	"C#7" => { Ok(3520.0 * 2.0_f64.powf(4.0/12.0)) },
	"Db7" => { Ok(3520.0 * 2.0_f64.powf(4.0/12.0)) },
	 "D7" => { Ok(3520.0 * 2.0_f64.powf(5.0/12.0)) },
	"D#7" => { Ok(3520.0 * 2.0_f64.powf(6.0/12.0)) },
	"Eb7" => { Ok(3520.0 * 2.0_f64.powf(6.0/12.0)) },
	 "E7" => { Ok(3520.0 * 2.0_f64.powf(7.0/12.0)) },
	 "F7" => { Ok(3520.0 * 2.0_f64.powf(8.0/12.0)) },
	"F#7" => { Ok(3520.0 * 2.0_f64.powf(9.0/12.0)) },
	"Gb7" => { Ok(3520.0 * 2.0_f64.powf(9.0/12.0)) },
	 "G7" => { Ok(3520.0 * 2.0_f64.powf(10.0/12.0)) },
	"G#7" => { Ok(3520.0 * 2.0_f64.powf(11.0/12.0)) },
	"Ab7" => { Ok(3520.0 * 2.0_f64.powf(11.0/12.0)) },
	 "A8" => { Ok(7040.0) },
	"A#8" => { Ok(7040.0 * 2.0_f64.powf(1.0/12.0)) },
        "Bb8" => { Ok(7040.0 * 2.0_f64.powf(1.0/12.0)) },
	 "B8" => { Ok(7040.0 * 2.0_f64.powf(2.0/12.0)) },
	 "C8" => { Ok(7040.0 * 2.0_f64.powf(3.0/12.0)) },
	"C#8" => { Ok(7040.0 * 2.0_f64.powf(4.0/12.0)) },
	"Db8" => { Ok(7040.0 * 2.0_f64.powf(4.0/12.0)) },
	 "D8" => { Ok(7040.0 * 2.0_f64.powf(5.0/12.0)) },
	"D#8" => { Ok(7040.0 * 2.0_f64.powf(6.0/12.0)) },
	"Eb8" => { Ok(7040.0 * 2.0_f64.powf(6.0/12.0)) },
	 "E8" => { Ok(7040.0 * 2.0_f64.powf(7.0/12.0)) },
	 "F8" => { Ok(7040.0 * 2.0_f64.powf(8.0/12.0)) },
	"F#8" => { Ok(7040.0 * 2.0_f64.powf(9.0/12.0)) },
	"Gb8" => { Ok(7040.0 * 2.0_f64.powf(9.0/12.0)) },
	 "G8" => { Ok(7040.0 * 2.0_f64.powf(10.0/12.0)) },
	"G#8" => { Ok(7040.0 * 2.0_f64.powf(11.0/12.0)) },
	"Ab8" => { Ok(7040.0 * 2.0_f64.powf(11.0/12.0)) },
	_ => { Err(ParseError) }
    }
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
	Self{wave_form: WaveForm::Square, volume: 0.25, frequency: 440.0, duration: 0.25, time: 0.0}
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
    // let _ = std::io::stdout().write(&[82, 73, 70, 70, 62, 250, 0, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, CHANNEL_COUNT, 0, 128, 62, 0, 0, 0, 125, 0, 0, 2, 0, 16, 0, 76, 73, 83, 84, 26, 0, 0, 0, 73, 78, 70, 79, 73, 83, 70, 84, 14, 0, 0, 0, 76, 97, 118, 102, 54, 48, 46, 49, 54, 46, 49, 48, 48, 0, 100, 97, 116, 97, 248, 249]);
    let mut data_buffer: Vec<u8> = Vec::<u8>::new();
    
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
				"pitch" => { default.frequency = pitch_to_frequency(halves[1].as_str()).unwrap(); }, // Placeholder
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
				"pitch" => { note.frequency = pitch_to_frequency(halves[1].as_str()).unwrap(); }, // Placeholder
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
	data_buffer.extend_from_slice(&sample_data(audio_accumulator));
    }

    let _ = std::io::stdout().write(&[82, 73, 70, 70]);
    let _ = std::io::stdout().write(&(data_buffer.len() as u32 + 68).to_le_bytes());
    let _ = std::io::stdout().write(&[87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, CHANNEL_COUNT, 0, 128, 62, 0, 0, 0, 125, 0, 0, 2, 0, 16, 0, 76, 73, 83, 84, 26, 0, 0, 0, 73, 78, 70, 79, 73, 83, 70, 84, 14, 0, 0, 0, 76, 97, 118, 102, 54, 48, 46, 49, 54, 46, 49, 48, 48, 0, 100, 97, 116, 97]);
    let _ = std::io::stdout().write(&(data_buffer.len() as u32).to_le_bytes());
    let _ = std::io::stdout().write(&data_buffer);
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
