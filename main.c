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
  if (seq) {
    int i;
    printf(">%s:%d-%d\n", name, start, end + 1);
    for (i = 0; i < end - start + 1; ++i) {
      if (i != 0 && !(i % 50))
	putc('\n', stdout);
      putc(seq[i], stdout);
    }
    putc('\n', stdout);
  }

  /* free resources */
  if (seq) free(seq);
  twobit_close(tb);

  return EXIT_SUCCESS;
}
