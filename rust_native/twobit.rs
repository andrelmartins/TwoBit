// twobit module containing a native implementation of the twobit query library
#![crate_id = "twobit#0.2"]
#![crate_type = "dylib"]
#![feature(macro_rules)]

extern crate native;
extern crate rustrt;

use std::os::MemoryMap;
use std::collections::hashmap::HashMap;
use std::io::{ IoResult, IoError };

use rustrt::rtio::RtioFileStream;

struct Block { start: u32, length: u32 }

struct Sequence {
	n_dna_bases: u32,
	unk_blocks: Vec<Block>,
	mask_blocks: Vec<Block>,
	dna_start: *mut u8
}

impl Sequence {
	fn string(&self, start: u32, end: u32) -> String {
		let mut rsize = (end - start + 1) as uint;
		let mut end = end;
		let mut result = String::from_char(rsize, 'N');
		
		if end >= self.n_dna_bases {
			end = self.n_dna_bases - 1;
			rsize = (end - start + 1) as uint;
		}

		unsafe {
			let bvec = result.as_mut_bytes();

			/* fill sequence */
			let mut block = self.dna_start.offset( (start / 4) as int);
			let mut offset = (start % 4) as uint;
			
			for i in range(0, rsize) {
				bvec[i] = byte_to_base(*block, offset) as u8;

				offset = offset + 1;
				if offset == 4 {
					offset = 0;
					block = block.offset(1);
				}
			}
		
			/* fill in Ns */
			for item in self.unk_blocks.iter() {
				let mut bstart = item.start;
				let mut bsize = item.length;
				let bend = bstart + bsize - 1;
			
				if bstart <= end && bend >= start {
					if bstart < start {
						bsize = bsize - (start - bstart);
						bstart = start;
					}
				
					let mut j = 0;
					let mut k = bstart;
					while j < bsize && k <= end {
						bvec[(k - start) as uint] = 'N' as u8;
				
						j = j + 1;
						k = k + 1;
					}
				}
			}
		}
		
		return result;
	}
}

pub struct TwoBit {
	seqs: HashMap<String, Sequence>,
	
	#[allow(dead_code)]
	data: MemoryMap // this is needed to keep the memory map alive
}

macro_rules! try_rt(
    ($e:expr) => (match $e { Ok(e) => e, Err(rustrt::rtio::IoError{code: code, extra: _, detail: _}) => return Err(IoError::from_errno(code, true)) })
)

fn mmap_read_u32(ptr: * mut u8, offset: uint) -> u32 {
	return unsafe { 
		let tmp : *u32 = std::mem::transmute(ptr.offset(offset as int) as *[u8, ..4]);
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
			let name = unsafe { String::from_raw_parts(name_size as uint, name_size as uint, data.offset((header_start + 1) as int)) };
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
	
	pub fn new(filename: &str) -> IoResult<TwoBit> {
		// open file
		let mut fh = try_rt!(native::io::file::open(&filename.to_c_str(), rustrt::rtio::Open, rustrt::rtio::Read));
		let fs = try_rt!(fh.fstat());
	
		// build memory map
		let mmap = match MemoryMap::new(fs.size as uint, [ std::os::MapReadable, std::os::MapFd(fh.fd())]) {
			Ok(val) => val,
			Err(_) => return Err(IoError{kind: std::io::OtherIoError, desc: "Memory map failed!", detail: None})
		};
		
		// TODO: add madvise call 
		
		
		// validate header
		let val = mmap_read_u32(mmap.data, 0);
		
		if val != 0x1A412743 {
			return Err(IoError { kind: std::io::OtherIoError, desc: "Invalid signature or wrong architecture.", detail: None });
		}
		
		let val = mmap_read_u32(mmap.data, 4);
		if val != 0 {
			return Err(IoError { kind: std::io::OtherIoError, desc: "Unknown file version.", detail: None });
		} // TODO: actually report version found
		
		let n_sequences = mmap_read_u32(mmap.data, 8);
		if n_sequences == 0 {
			return Err(IoError { kind: std::io::OtherIoError, desc: "Zero sequence count.", detail: None });
		}
		
		let val = mmap_read_u32(mmap.data, 12);
		if val != 0 {
			return Err(IoError { kind: std::io::OtherIoError, desc: "Reserved bytes not zero.", detail: None });
		} // TODO: actually report value found
		
		// parse index
		let index = mmap_read_index(mmap.data, n_sequences);
	
		return Ok(TwoBit { seqs: index, data: mmap });
	}
	
	pub fn sequence(&self, chrom: &str, start: u32, end: u32) -> Option<String> {
		match self.seqs.find(&String::from_str(chrom)) {
			Some(ref seq) => Some(seq.string(start, end)),
			None => None
		}	
	}
	
	pub fn sequence_len(&self, chrom: &str) -> Option<u32> {
		match self.seqs.find(&String::from_str(chrom)) {
			Some(&Sequence{ n_dna_bases: n, ..}) => Some(n),
			None => None
		}	
	}

	pub fn sequence_names<'a>(&'a self) -> Vec<&'a String> {
		self.seqs.keys().collect()
	}
	
	pub fn base_frequencies(&self, chrom: &str) -> Option<[f64, ..4]> {
		match self.seqs.find(&String::from_str(chrom)) {
			Some(ref seq) => {
				let seqstr = seq.string(0, seq.n_dna_bases - 1);
				let mut counts = [0f64, 0.0, 0.0, 0.0];
				
				for c in seqstr.as_slice().chars() {
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
