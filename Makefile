all: twoBitFreq test twobit.so

twoBitFreq: main_freq.c twobit.c twobit.h
	gcc -o twoBitFreq -Wall -g main_freq.c twobit.c 

test: main.c twobit.c twobit.h
	gcc -o test -Wall -g main.c twobit.c

twobit.so: twobit_R.c twobit.h twobit.c
	R CMD SHLIB twobit.c twobit_R.c
