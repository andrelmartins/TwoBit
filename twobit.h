#ifndef TWOBIT_H
#define TWOBIT_H

typedef struct twobit_ TwoBit;

TwoBit * twobit_open(const char * filename);
void twobit_close(TwoBit * ptr);

int twobit_sequence_size(TwoBit * ptr, const char * name);

/* start and end are _zero_ based */
char * twobit_sequence(TwoBit * ptr, const char * name, int start, int end);

/* return base frequencies: alphabet = ACGT */
double * twobit_base_frequencies(TwoBit * ptr, const char * name);

/* return sequence names (NULL terminated, caller must free memory) */
char ** twobit_sequence_names(TwoBit * ptr);
#endif
