// Implement twoBitFreq in Rust
extern crate twobit;

use std::os;
use twobit::TwoBit;
use std::error::Error;
use std::io;

fn main() {
	let args = os::args();

	match args.as_slice() {
		[ _, ref filename, ref chrom ] => {
			let tb = TwoBit::new(filename);
			
			match tb {
				Ok(tbv) => {
					match tbv.base_frequencies(chrom) {
						Some(freqs) => 
							println!("{} base frequencies (ACGT): {} {} {} {}",
							  chrom, freqs[0], freqs[1], freqs[2], freqs[3]),
						None => println!("Unknown sequence: {}", chrom),
					}	
				},
				Err(ioerr) => println!("{}: {}", ioerr.description(), filename)
			}
		},
		[ ref prog, ..] => println!("Usage: {} <2bit filename> <name> ", prog),
		_ => panic!()
	}
}
