// test main
extern crate twobit;
use std::os;
use twobit::TwoBit;
use std::io::Error;

fn print_sequence(seq: &str) {
   let mut i = 0i32;

   for base in seq.chars() {
	 if i != 0 && (i % 50) == 0 {
	   println!("");
	 }
	 print!("{}", base);
	 i = i + 1;
   }
   println!("");
}

fn main() {
	let args = os::args();

	match args.as_slice() {
		[ _, ref filename, ref chrom, ref start, ref end ] => {
		
			let start = start.parse::<u32>().ok().expect("Invalid start coordinate");
			let end = end.parse::<u32>().ok().expect("Invalid end coordinate");
		
			let tb = TwoBit::new(filename.as_slice());
			
			match tb {
				Ok(tbv) => {
					// get chromosome size
					match tbv.sequence_len(chrom.as_slice()) {
						Some(n) => println!("{}: size = {}", chrom, n),
						None => {
							println!("unknown sequence: {}", chrom);
							return;
						}
					};
				
					let seq = tbv.sequence(chrom.as_slice(), start, end);
					match seq {
						Some(seqstr) => {
							println!(">{}:{}-{}", chrom, start, end + 1);
							//println!("{}", seqstr);
							print_sequence(seqstr.as_slice());
						},
						None => println!("nothing")
					}; 
				},
				Err(ioerr) => println!("{}: {}", ioerr.description(), filename)
			}
		},
		[ ref prog, ..] => println!("Usage: {} <2bit filename> <name> <start> <end>", prog),
		_ => panic!()
	}
}
