#![crate_name = "twobit"]
#![crate_type = "dylib"]
#![feature(macro_rules)]
#![feature(associated_types)]

//! Implements the TwoBit struct giving read access to 2bit files in the format
//! used by the UCSC Genome Browser (details at: http://genome.ucsc.edu/FAQ/FAQformat.html#format7).


use std::os::MemoryMap;
use std::collections::HashMap;
use std::io::{ IoResult, IoError };
use std::iter::FromIterator;
use std::os::unix::AsRawFd;

#[derive(Show)]
struct Block { start: u32, length: u32 }

struct Sequence {
	n_dna_bases: u32,
	unk_blocks: Vec<Block>,
	#[allow(dead_code)]
	mask_blocks: Vec<Block>,
	dna_start: *mut u8
}

impl Sequence {
	fn range<'a>(&'a self, start: u32, end: u32) -> SeqRange<'a> {
		let mut rsize = (end - start + 1) as uint;
		let mut end = end;

		if end >= self.n_dna_bases {
			let rest = rsize - self.n_dna_bases as uint;
			end = self.n_dna_bases - 1;
			rsize = (end - start + 1) as uint;

			// seq range chained with an iterator that repeats the same value
			SeqRange { 
				rsize: rsize, 
				ptr: unsafe { self.dna_start.offset( (start / 4) as int) },
				idx: 0u,
				offset: (start % 4) as uint,
				unk_blocks: &self.unk_blocks,
				ub_exhausted: self.unk_blocks.len() == 0,
				ub_idx: 0,
				ub_start: if self.unk_blocks.len() > 0 { self.unk_blocks[0].start as uint } else { 0 },
				ub_end: if self.unk_blocks.len() > 0 { (self.unk_blocks[0].start + self.unk_blocks[0].length - 1) as uint } else { 0 },
				n_more: rest
			}
		} else {
			SeqRange { 
				rsize: rsize, 
				ptr: unsafe { self.dna_start.offset( (start / 4) as int) },
				idx: 0u,
				offset: (start % 4) as uint,
				unk_blocks: &self.unk_blocks,
				ub_exhausted: self.unk_blocks.len() == 0,
				ub_idx: 0,
				ub_start: if self.unk_blocks.len() > 0 { self.unk_blocks[0].start as uint } else { 0 },
				ub_end: if self.unk_blocks.len() > 0 { (self.unk_blocks[0].start + self.unk_blocks[0].length - 1) as uint } else { 0 },
				n_more: 0
			}
		}
	}

	fn string(&self, start: u32, end: u32) -> String {
		FromIterator::from_iter(self.range(start, end))
	}
}

/// Sequence range iterator
pub struct SeqRange<'a> {
	rsize: uint,
	ptr: * mut u8,
	idx: uint,
	offset: uint,
	unk_blocks: &'a Vec<Block>,
	ub_exhausted: bool,
	ub_idx: uint,
	ub_start: uint,
	ub_end: uint,
	n_more: uint
}

impl<'a> SeqRange<'a> {
	#[inline]
	fn increment_idx(&mut self) {
		self.idx = self.idx + 1;
		self.offset = self.offset + 1;
		if self.offset == 4 {
			self.offset = 0;
			self.ptr = unsafe { self.ptr.offset(1) };
		}
	}
}

impl<'a> Iterator for SeqRange<'a> {
	type Item = char;

	fn next(&mut self) -> Option<char> {
		if self.idx == self.rsize {
			if self.n_more > 0 {
				self.n_more = self.n_more - 1;
				return Some('N');
			}
			return None;
		} else {
			unsafe {
				// are we within a block?
				if !self.ub_exhausted {
					loop {
						if self.idx > self.ub_end {
							self.ub_idx = self.ub_idx + 1;
							if self.ub_idx == self.unk_blocks.len() {
								self.ub_exhausted = true;
								break;
							} else {
								self.ub_start = self.unk_blocks[self.ub_idx].start as uint;
								self.ub_end = self.ub_start + self.unk_blocks[self.ub_idx].length as uint - 1;
							}
						} else if self.idx >= self.ub_start {
							self.increment_idx();

							return Some('N');
						} else {
							// outside block, so continue to data collection
							break;
						}
					}
				}

				// no, so collect data
				let result = Some(byte_to_base(*self.ptr, self.offset));

				self.increment_idx();

				return result;
			}
		}
	}

	fn size_hint(&self) -> (uint, Option<uint>) {
		// lower and upper bound (same)
		let bound = self.rsize - self.idx - self.n_more;
		(bound, Some(bound))
	}
}

/// TwoBit type, represents a 2bit object in the UCSC Genome Browser format
pub struct TwoBit {
	seqs: HashMap<String, Sequence>,
	
	#[allow(dead_code)]
	data: MemoryMap // this is needed to keep the memory map alive
}

macro_rules! try_rt(
    ($e:expr) => (match $e { Ok(e) => e, Err(rustrt::rtio::IoError{code: code, extra: _, detail: _}) => return Err(IoError::from_errno(code, true)) })
);

fn mmap_read_u32(ptr: * mut u8, offset: uint) -> u32 {
	return unsafe { 
		let tmp : *const u32 = std::mem::transmute(ptr.offset(offset as int) as *const [u8; 4]);
		*tmp };
}

fn mmap_read_u8(ptr: * mut u8, offset: uint) -> u8 {
	return unsafe {
		*ptr.offset(offset as int)
	};
}

fn read_blocks(data: * mut u8, offset: u32) -> (Vec<Block>, u32) {
	let len = mmap_read_u32(data, offset as uint);
	let mut vec = Vec::<Block>::new();
	
	if len > 0 {
		let off1 = offset + 4;
		let off2 = offset + 4 + len*4;
		
		for i in range(0, len) {
			let start = mmap_read_u32(data, (off1 + i*4) as uint);
			let size = mmap_read_u32(data, (off2 + i*4) as uint);
			
			vec.push(Block{ start: start, length: size });
		}
	}
	
	return (vec, offset + 4 + 2*len*4);
}

fn mmap_read_index(data: *mut u8, count: u32) -> HashMap<String, Sequence> {
		let mut index = HashMap::with_capacity(count as uint);
	
		let mut header_start = 16u;
			
		for _ in range(0, count) {
			let name_size = mmap_read_u8(data, header_start);
			let name = unsafe {
				let slice: &mut [u8] = std::mem::transmute(std::raw::Slice { data: data.offset((header_start + 1) as int), len: name_size as uint });
				let strslice = std::str::from_utf8_unchecked(slice);				
				String::from_str(strslice)
			};
			
			let offset = mmap_read_u32(data, header_start + 1 + name_size as uint);
			
			// get actual info
			let dna_size = mmap_read_u32(data, offset as uint);

			// unknown blocks
			let (unk_blocks, offset) = read_blocks(data, offset + 4);
			
			// masked blocks
			let (mask_blocks, offset) = read_blocks(data, offset);
			
			// actual pointer to DNA data
			let dna_ptr = unsafe { data.offset((offset + 4) as int) }; // + reserved
			
			index.insert(name, Sequence { n_dna_bases: dna_size, unk_blocks: unk_blocks, mask_blocks: mask_blocks, dna_start: dna_ptr });			
			
			header_start = header_start + 1 + name_size as uint + 4;
		}
	
		return index;
}

fn byte_to_base(value: u8, offset: uint) -> char {
	let bases = ['T', 'C', 'A', 'G'];
	let rev_offset = 3 - offset;
	let mask = 3 << (rev_offset * 2);
	let idx = (value & mask) >> (rev_offset * 2);
	
	return bases[idx as uint];
}

impl TwoBit {
	
	/// Create a new TwoBit object from the supplied filename
	///
	/// # Arguments
	///
	/// - filename - string slice with the path to the 2bit file
	pub fn new(filename: &str) -> IoResult<TwoBit> {
		// TODO: revise interface to take a "File" instance instead of a filename
	
		// open file
		let fh = try!(std::io::File::open(&Path::new(filename)));
		let fs = try!(fh.stat());
	
		// build memory map
		let mmap = match MemoryMap::new(fs.size as uint, &[ std::os::MapOption::MapReadable, std::os::MapOption::MapFd(fh.as_raw_fd())]) {
			Ok(val) => val,
			Err(_) => return Err(IoError{kind: std::io::OtherIoError, desc: "Memory map failed!", detail: None})
		};
		
		// TODO: add madvise call 
		
		
		// validate header
		let val = mmap_read_u32(mmap.data(), 0);
		
		if val != 0x1A412743 {
			return Err(IoError { kind: std::io::OtherIoError, desc: "Invalid signature or wrong architecture.", detail: None });
		}
		
		let val = mmap_read_u32(mmap.data(), 4);
		if val != 0 {
			return Err(IoError { kind: std::io::OtherIoError, desc: "Unknown file version.", detail: None });
		} // TODO: actually report version found
		
		let n_sequences = mmap_read_u32(mmap.data(), 8);
		if n_sequences == 0 {
			return Err(IoError { kind: std::io::OtherIoError, desc: "Zero sequence count.", detail: None });
		}
		
		let val = mmap_read_u32(mmap.data(), 12);
		if val != 0 {
			return Err(IoError { kind: std::io::OtherIoError, desc: "Reserved bytes not zero.", detail: None });
		} // TODO: actually report value found
		
		// parse index
		let index = mmap_read_index(mmap.data(), n_sequences);
	
		return Ok(TwoBit { seqs: index, data: mmap });
	}
	
	/// Collect the sequence for the range [start, end]
	///
	/// Note that when ranges exceed the recorded information contained in the 2bit file
	/// excess is padded with 'N's.
	///
	/// # Arguments
	///
	/// - chrom - sequence name, typically the chromosome name
	/// - start - zero based start coordinate for range
	/// - end - zero based end coordinate (inclusive) for range
	pub fn sequence(&self, chrom: &str, start: u32, end: u32) -> Option<String> {
		match self.seqs.get(&String::from_str(chrom)) {
			Some(ref seq) => Some(seq.string(start, end)),
			None => None
		}
	}

	/// Iterator on the sequence for the range [start, end]
	///
	/// Note that when ranges exceed the recorded information contained in the 2bit file
	/// excess is padded with 'N's.
	///
	/// # Arguments
	///
	/// - chrom - sequence name, typically the chromosome name
	/// - start - zero based start coordinate for range
	/// - end - zero based end coordinate (inclusive) for range
	pub fn sequence_iter<'a>(&'a self, chrom: &str, start: u32, end: u32) -> Option<SeqRange<'a>> {
		match self.seqs.get(&String::from_str(chrom)) {
			Some(ref seq) => Some(seq.range(start, end)),
			None => None
		}
	}
	
	/// Retrieve the length (number of bases) of a given sequence
	///
	/// # Arguments
	///
	/// - chrom - sequence name, typically the chromosome name
	pub fn sequence_len(&self, chrom: &str) -> Option<u32> {
		match self.seqs.get(&String::from_str(chrom)) {
			Some(&Sequence{ n_dna_bases: n, ..}) => Some(n),
			None => None
		}	
	}

	/// Retrieve the names of the sequences contained in this file
	pub fn sequence_names<'a>(&'a self) -> Vec<&'a String> {
		self.seqs.keys().collect()
	}

	/// Compute the nucleotide frequencies (ACGT) of a given sequence
	///
	/// # Arguments
	///
	/// - chrom - sequence name, typically the chromosome name
	pub fn base_frequencies(&self, chrom: &str) -> Option<[f64; 4]> {
		match self.seqs.get(&String::from_str(chrom)) {
			Some(ref seq) => {
				let mut counts = [0f64, 0.0, 0.0, 0.0];

				for c in seq.range(0, seq.n_dna_bases - 1) {
					match c {
						'A' => counts[0] = counts[0] + 1.0,
						'C' => counts[1] = counts[1] + 1.0,
						'G' => counts[2] = counts[2] + 1.0,
						'T' => counts[3] = counts[3] + 1.0,
						_ => {}
					}
				}
				
				let sum = counts[0] + counts[1] + counts[2] + counts[3];
				counts[0] = counts[0] / sum;
				counts[1] = counts[1] / sum;
				counts[2] = counts[2] / sum;
				counts[3] = counts[3] / sum;
				
				Some(counts)
			}
			None => None
		}
	}
}

/// Set of useful nucleotide sequence manipulation operations.
pub trait DNAOps {

	/// Obtain reverse complement of nucleotide sequence
	fn reverse_complement(&self) -> Self;
	
	/// Convert a DNA string into an integer vector
	///
	/// # Arguments
	///
	/// - offset - starting value for DNA nucleotides (A = offset, C = offset + 1, G = offset + 2, T = offset + 3, else offset + 4)
	fn into_numeric(&self, offset: u8) -> Vec<u8>;
}

impl DNAOps for String {

	fn reverse_complement(&self) -> String {
		// create a new string by iterating over the chars of the old one
		let mut result = String::with_capacity(self.len());
		
		for x in self.chars().map(|base|
		  match base {
		  	'a' => 't',
		  	'A' => 'T',
		  	'c' => 'g',
		  	'C' => 'G',
		  	'g' => 'c',
		  	'G' => 'C',
		  	't' => 'a',
		  	'T' => 'A',
		  	_ => 'N'
		  }) {
		  result.push(x);
		} 
		 
		// use as_mut_vec to reverse the resulting string inplace
		unsafe {
			let vec = result.as_mut_vec();
			vec.reverse();
		}
		return result;
	}
	
	fn into_numeric(&self, offset: u8) -> Vec<u8> {
		self.chars().map(|base: char|
		 match base {
		 	'a' | 'A' => offset,
		 	'c' | 'C' => offset + 1,
		 	'g' | 'G' => offset + 2,
		 	't' | 'T' => offset + 3,
		 	_ => offset + 4
		 }).collect::<Vec<u8>>()
	}
}
