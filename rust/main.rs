extern crate twobit;
use std::os;
use std::from_str::from_str;

fn print_sequence(seq: &str) {
   let mut i = 0;

   for base in seq.chars() {
     if i != 0 && (i % 50) == 0 {
       println!("");
     }
     print!("{}", base);
     i = i + 1;
   }
   println!("");
}

fn process_request(tb: &twobit::TwoBit, chrom: &str, start: i32, end: i32) {
 // get 
 match tb.sequence_len(chrom) {
   Some(n) => println!("{}: size = {}", chrom, n),
   None => {
     println!("unknown sequence: {}", chrom);
     return;
   }
 };

 let seq = tb.sequence(chrom, start, end);
 match seq {
   Some(seqstr) => {
     println!(">{}:{}-{}", chrom, start, end + 1);
     //println!("{}", seqstr);
     print_sequence(seqstr.as_slice());
   },
   None => println!("nothing")
 }; 
}

fn main() {
   let args = os::args();

   if args.len() == 5 {
     // parse arguments
     let filename = args.get(1).as_slice();
     let chrom = args.get(2).as_slice();
     let start = from_str::<i32>(args.get(3).as_slice());
     let end = from_str::<i32>(args.get(4).as_slice());

     let start_value = match start {
       Some(value) => value,
       None => {
         println!("Usage: {} <2bit filename> <name> <start> <end>", args.get(0));
       	 return;
       }
     };
     let end_value = match end {
       Some(value) => value,
       None => {
         println!("Usage: {} <2bit filename> <name> <start> <end>", args.get(0));
       	 return;
       }
     };

     let tb = twobit::TwoBit::new(filename);

     match tb {
      Some(ref tbr) => process_request(tbr, chrom, start_value, end_value),
      None => println!("file not found, or open failed!"),
     };
   } else {
     println!("Usage: {} <2bit filename> <name> <start> <end>", args.get(0));
   }
}
