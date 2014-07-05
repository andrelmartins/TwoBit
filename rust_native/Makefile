all: main twoBitFreq

main: twobit.rs main.rs
	rustc twobit.rs
	rustc main.rs -L .

twoBitFreq: twoBitFreq.rs twobit.rs
	rustc twobit.rs
	rustc twoBitFreq.rs -L .
