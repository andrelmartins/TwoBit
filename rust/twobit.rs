// `twobit.rs` - contains body of module 'twobit' in the crate root
#![crate_id = "twobit#0.1"]
#![crate_type = "dylib"]

extern crate libc;
use std::ptr;
use std::str;

#[no_mangle]
pub extern "C" fn test() {
  println!("hello from Rust!");
}

// first version implemented as a wrapper around the C lib
//

#[link(name = "twobit")]
extern {
       fn twobit_open(filename: *libc::c_char) -> *libc::c_void;
       fn twobit_close(ptr: *libc::c_void) -> *libc::c_void;
       fn twobit_sequence(ptr: *libc::c_void, chrom: *libc::c_char, start: libc::c_int, end: libc::c_int) -> *libc::c_char;
       fn twobit_sequence_size(ptr: *libc::c_void, chrom: *libc::c_char) -> libc::c_int;
}

pub struct TwoBit {
       ptr: *libc::c_void
}

impl TwoBit {
     pub  fn new(filename: &str) -> Option<TwoBit> {
        filename.with_c_str(|c_buffer| {
	  unsafe {
	   let cptr = twobit_open(c_buffer);
	   
	   if cptr == ptr::null() {
	     None
	   } else {
	     Some(TwoBit { ptr: cptr })
	   }
          }
	})
     }

     pub fn sequence(&self, chrom: &str, start: i32, end: i32) -> Option<~str> {
     	 chrom.with_c_str(|c_buffer| {
	   unsafe {
	     let cptr = twobit_sequence(self.ptr, c_buffer, start, end);
	     if cptr.is_null() {
	       None
	     } else {
	       let res = Some(str::raw::from_c_str(cptr));
	       libc::free(cptr as *mut libc::c_void);
	       res
	     }
	   }
	 })
     }

     pub fn sequence_len(&self, chrom: &str) -> Option<i32> {
     	 chrom.with_c_str(|c_buffer| {
	   unsafe {
	     match twobit_sequence_size(self.ptr, c_buffer) {
	       -1 => None,
	       n => Some(n)
	     }
           }
	 })
     }
}

impl Drop for TwoBit {
    fn drop(&mut self) {
      unsafe {
        twobit_close(self.ptr);
      }
    }
}
