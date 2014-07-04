all: main

main: main.rs twobit.rs
	rustc twobit.rs
	rustc main.rs -L .

test: main
	gcc test.c -ltwobit-984fb1d7-0.1 -ltwobit  -L. -o test
