#include "twobit.h"
#include <stdlib.h>
#include <stdio.h>

int main(int argc, char ** argv) {
  char * filename;
  TwoBit * tb;
  char * name;
  double * freqs;
  int i;

  if (argc != 3) {
    printf("twoBitFreq - Compute base frequencies of a given sequence\n\n");
    printf("Usage: %s <2bit filename> <name>\n", argv[0]);
    return EXIT_FAILURE;
  }

  filename = argv[1];
  name = argv[2];

  tb = twobit_open(filename);
  if (tb == NULL) {
    fprintf(stderr, "Failed to open: %s\n", filename);
    return EXIT_FAILURE;
  }

  
  freqs = twobit_base_frequencies(tb, name);

  printf("%s base frequencies (ACGT):", name);
  for (i = 0; i < 4; ++i)
    printf(" %g", freqs[i]);
  putc('\n', stdout);

  /* free resources */
  free(freqs);
  twobit_close(tb);

  return EXIT_SUCCESS;
}
