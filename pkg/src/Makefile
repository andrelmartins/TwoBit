all: twobit.so

twobit.so: twobit_R.c twobit.h twobit.c
	R CMD SHLIB twobit.c twobit_R.c

clean:
	rm -f *.o *.so
