all: twoBitFreq test twobit.so

twoBitFreq: main_freq.c twobit.c twobit.h
	gcc -o twoBitFreq -Wall -g main_freq.c twobit.c 

test: main.c twobit.c twobit.h
	gcc -o test -Wall -g main.c twobit.c

twobit.so: twobit.c
	gcc -o twobit.so -shared twobit.c -fPIC

twoBit.pkg:
	rm -Rf pkg.roxygen
	(R CMD roxygen pkg)||(./Roxygen pkg)
	R CMD INSTALL pkg.roxygen
