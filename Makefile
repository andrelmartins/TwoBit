all: twoBitFreq test

twoBitFreq: main_freq.c twobit.c twobit.h
	gcc -o twoBitFreq -Wall -g main_freq.c twobit.c 

test: main.c twobit.c twobit.h
	gcc -o test -Wall -g main.c twobit.c

twoBit.so: twobit.c
	gcc -o twobit.so -shared twobit.c

twoBit.pkg:
	rm -Rf pkg.roxygen
	R CMD roxygen pkg
	R CMD INSTALL pkg.roxygen
