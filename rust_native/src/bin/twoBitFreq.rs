// Implement twoBitFreq in Rust
extern crate twobit;

use twobit::TwoBit;
use std::error::Error;

fn main() {
	let mut args = std::env::args();

	if args.len() != 3 {
		println!("Usage: {} <2bit filename> <name> ", args.next().unwrap());
	} else {
		let mut args = args.skip(1);
		let filename = args.next().unwrap();
		let chrom = args.next().unwrap();
		let tb = TwoBit::new(&filename);
			
		match tb {
			Ok(tbv) => {
				match tbv.base_frequencies(&chrom) {
					Some(freqs) => 
						println!("{} base frequencies (ACGT): {} {} {} {}",
						  chrom, freqs[0], freqs[1], freqs[2], freqs[3]),
					None => println!("Unknown sequence: {}", chrom),
				}	
			},
			Err(ioerr) => println!("{}: {}", ioerr.description(), filename)
		}
	}
}
