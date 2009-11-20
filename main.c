#include "twobit.h"
#include <stdlib.h>
#include <stdio.h>

int main(int argc, char ** argv) {
  char * filename;
  TwoBit * tb;
  int start, end;
  char * seq;
  char * name;

  if (argc != 5) {
    printf("Usage: %s <2bit filename> <name> <start> <end>\n", argv[0]);
    return EXIT_FAILURE;
  }

  filename = argv[1];
  name = argv[2];
  start = atoi(argv[3]);
  end = atoi(argv[4]);

  tb = twobit_open(filename);
  if (tb == NULL) {
    fprintf(stderr, "Failed to open: %s\n", filename);
    return EXIT_FAILURE;
  }

  printf("%s: size = %d\n", name, twobit_sequence_size(tb, name));

  seq = twobit_sequence(tb, name, start, end);
  if (seq)
    printf("%s: %s\n", name, seq);

  /* free resources */
  if (seq) free(seq);
  twobit_close(tb);

  return EXIT_SUCCESS;
}
