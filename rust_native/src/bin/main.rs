// test main
extern crate twobit;
use twobit::TwoBit;

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
	let mut args = std::env::args();

	if args.len() != 5 {
		println!("Usage: {} <2bit filename> <name> <start> <end>", args.next().unwrap());
	} else {
		let mut args = args.skip(1);
		let filename = args.next().unwrap();
		let chrom = args.next().unwrap();
		let start = args.next().unwrap();
		let end = args.next().unwrap();
		
		let start = start.parse::<u32>().ok().expect("Invalid start coordinate");
		let end = end.parse::<u32>().ok().expect("Invalid end coordinate");
	
		let tb = TwoBit::new(&filename);
		
		match tb {
			Ok(tbv) => {
				// get chromosome size
				match tbv.sequence_len(&chrom) {
					Some(n) => println!("{}: size = {}", chrom, n),
					None => {
						println!("unknown sequence: {}", chrom);
						return;
					}
				};
			
				let seq = tbv.sequence(&chrom, start, end);
				match seq {
					Some(seqstr) => {
						println!(">{}:{}-{}", chrom, start, end + 1);
						//println!("{}", seqstr);
						print_sequence(&seqstr);
					},
					None => println!("nothing")
				}; 
			},
			Err(ioerr) => println!("{}: {}", ioerr, filename)
		}
	}
}
