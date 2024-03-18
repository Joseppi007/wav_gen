use std::io::Write;
use std::fs::File;

const SAMPLES_PER_SECOND: u128 = 16000;

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

fn sine_wave (virt_time: f64, amplitude: f64) -> f64 { // 1 Hz
    (1.0+( virt_time * std::f64::consts::TAU ).sin())*amplitude
}

fn square_wave (virt_time: f64, amplitude: f64) -> f64 { // 1 Hz
    if virt_time % 1.0 < 0.5 {
	return 0.0;
    } else {
	return 2.0*amplitude;
    }
}

fn triangle_wave (virt_time: f64, amplitude: f64) -> f64 { // 1 Hz
    if virt_time % 1.0 < 0.5 {
	virt_time * 4.0 * amplitude
    } else {
	(1.0-virt_time) * 4.0 * amplitude
    }
}

fn pulse_wave (virt_time: f64, amplitude: f64, ratio: f64) -> f64 { // 1 Hz
    if virt_time % 1.0 < 1.0 - ratio {
	0.0
    } else {
	2.0*amplitude
    }
}

fn saw_tooth_wave (virt_time: f64, amplitude: f64) -> f64 { // 1Hz
    return (virt_time % 1.0) * 2.0 * amplitude;
}

fn write_sample (amplitude: f64) -> () {
    let mut a: f64 = amplitude * 255.0;
    if a > 255.0 { a = 255.0; }
    if a < 0.0   { a = 0.0;   }
    let _ = std::io::stdout().write(&[ a as u8 ]);
}

fn print_wave( file: File ) -> () {
    let _ = std::io::stdout().write(&[82, 73, 70, 70, 36, 0, 0, 128, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1, 0, 128, 62, 0, 0, 0, 0, 0, 0, 1, 0, 8, 0, 100, 97, 116, 97]);

    for i in 0..16000 {
	write_sample( square_wave( scale_time_linear(i, 440.0), 0.5 ) );
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
