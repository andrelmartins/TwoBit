// Implement twoBitFreq in Rust
extern crate twobit;

use std::os;
use twobit::TwoBit;

fn main() {
	let args = os::args();

	match args.as_slice() {
		[ _, ref filename, ref chrom ] => {
			let tb = TwoBit::new(filename.as_slice());
			
			match tb {
				Ok(tbv) => {
					match tbv.base_frequencies(chrom.as_slice()) {
						Some(freqs) => 
							println!("{} base frequencies (ACGT): {} {} {} {}",
							  chrom, freqs[0], freqs[1], freqs[2], freqs[3]),
						None => println!("Unknown sequence: {}", chrom),
					}	
				},
				Err(std::io::IoError{ kind: _, desc: x, detail: _}) => println!("{}: {}", x, filename)
			}
		},
		[ ref prog, ..] => println!("Usage: {} <2bit filename> <name> ", prog),
		_ => fail!()
	}
}
