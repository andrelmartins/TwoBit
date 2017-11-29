#include "twobit.h"
#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <string.h>

/* 2bit file format http://genome.ucsc.edu/FAQ/FAQformat#format7 */

typedef unsigned int uint32;

struct twobit_index {
  struct twobit_index * next;
  char * name;

  int size;

  int n_blocks;
  uint32 * n_block_starts;
  uint32 * n_block_sizes;
  const unsigned char * sequence;
};

struct twobit_ {
  int fd;
  struct stat sb;
  const char * data; /* MMAP pointer */

  struct twobit_index * index;
};

/* returns sequence count if valid, -1 otherwise */
int validate_header(const char * data) {
  const uint32 * ptr = (const uint32*) data;
  int sequenceCount;
  
  /* signature */
  if (*ptr != 0x1A412743) {
    fprintf(stderr, "Invalid signature or wrong architecture.\n");
    return -1;
  }
  ++ptr;
  /* version */
  if (*ptr != 0) {
    fprintf(stderr, "Unknown file version: %d.\n", *ptr);
    return -1;
  }
  ++ptr;
  /* sequence count */
  sequenceCount = *ptr;
  ++ptr;
  /* reserved bytes */
  if (*ptr != 0) {
    fprintf(stderr, "Reserved bytes not zero: %d.\n", *ptr);
    return -1;
  }

  return sequenceCount;
}

static struct twobit_index * tbi_push(struct twobit_index * head, int size, const char * ptr, const char * data, uint32 offset) {
  struct twobit_index * res = (struct twobit_index*) malloc(sizeof(struct twobit_index));
  uint32 * rec;
  uint32 aux;

  res->next = head;
  res->name = strndup(ptr, size * sizeof(char));

  /* read record start */
  rec = (uint32*) (data + offset);

  /* size */
  res->size = *rec;
  ++rec;

  /* blocks of Ns */
  res->n_blocks = *rec;
  ++rec;
  if (res->n_blocks > 0) {
    res->n_block_starts = rec;
    rec += res->n_blocks;
    res->n_block_sizes = rec;
    rec += res->n_blocks;
  }

  /* skip masks */
  aux = *rec;
  rec += 2*aux + 1;

  /* skip reserved */
  ++rec;

  res->sequence = (unsigned char *) rec;

  return res;
}

static struct twobit_index * read_index(const char * data, int sequenceCount) {
  /* index has:
     unsigned char nameSize;
     char * name;
     uint32 offset;
  */
  unsigned char nameSize;
  uint32 offset;
  struct twobit_index * result = NULL;
  const char * start_file = data;

  /* skip header */
  data += sizeof(uint32)*4; /* header has 4 uint32 entries */
  
  /* read index entries */
  for (; sequenceCount > 0; --sequenceCount) {
    const char * name;
    nameSize = *data;
    ++data;
    name = data;
    data += nameSize;
    offset = *((uint32*) data);
    data += sizeof(uint32);

    /* build entry */
    result = tbi_push(result, (int) nameSize, name, start_file, offset);
  }

  return result;
}

void free_index(struct twobit_index * index) {
  struct twobit_index * ptr;

  while (index != NULL) {
    ptr = index->next;
    free(index->name);
    free(index);
    index = ptr;
  }
}

TwoBit * twobit_open(const char * filename) {
  int fd;
  struct stat sb;
  char * data;
  int sequenceCount;
  TwoBit * result;
  
  /* open file */
  fd = open(filename, O_RDONLY);
  if (fd == -1) {
    perror("twobit_open");
    return NULL;
  }

  /* get file size */
  if (fstat(fd, &sb) == -1) { /* To obtain file size */
    perror("fstat failed");
    return NULL;
  }

  /* mmap file */
  data = mmap(NULL, sb.st_size, PROT_READ, MAP_PRIVATE, fd, 0);
  if (data == MAP_FAILED) {
    perror("mmap failed");
    return NULL;
  }

  if (madvise(data, sb.st_size, MADV_RANDOM) == -1) {
    perror("advise failed");
    /* no need to exit */
  }

  /* validate header */
  if ((sequenceCount = validate_header(data)) < 0) {
    munmap((void*)data, sb.st_size);
    close(fd);
    return NULL;
  }

  /* fill structure */
  result = (TwoBit*) malloc(sizeof(TwoBit));
  result->fd = fd;
  result->sb = sb;
  result->data = data;
  result->index = read_index(data, sequenceCount);

  return result;
}

void twobit_close(TwoBit * ptr) {
  munmap((void*)ptr->data, ptr->sb.st_size);

  free_index(ptr->index);

  close(ptr->fd);

  free(ptr);
}

static struct twobit_index * find_sequence(struct twobit_index * index, const char * name) {
  while (index != NULL) {
    if (!strcmp(index->name, name))
      return index;
    index = index->next;
  }
  return NULL;
}

int twobit_sequence_size(TwoBit * ptr, const char * name) {
  struct twobit_index * seq = find_sequence(ptr->index, name);
  if (!seq) 
    return -1;

  return seq->size;
}

/* byte has three 2 bit bases, offset can be 0, 1, 2 or 3 */
char byte_to_base(unsigned char byte, int offset) {
  int rev_offset = 3 - offset;
  unsigned char mask = 3 << (rev_offset * 2);
  int idx = (byte & mask) >> (rev_offset * 2);
  char * bases = "TCAG";

  return bases[idx];
}

char * twobit_sequence(TwoBit * ptr, const char * name, int start, int end) {
  struct twobit_index * seq = find_sequence(ptr->index, name);
  int size, rsize;
  char * result, * seq_dest;
  int i;

  if (!seq) 
    return NULL;

  if (start > end)
    return NULL;

  size = seq->size;
  rsize = end - start + 1;
  seq_dest = result = (char*) malloc((rsize + 1) * sizeof(char));
  if(result == NULL) return NULL;
  memset(result, 'N', rsize * sizeof(char)); /* initialize */
  result[rsize] = '\0';

  if (end >= size) {
    end = size - 1;
    rsize = end - start + 1;
  }

  if (start < 0)
    {
      rsize += start;
      seq_dest -= start;
      start = 0;
    }
  if (end < 0)
    {
      rsize = 0;
      seq_dest = result;
      end = 0;
    };

  /* fill sequence */
  {
    const unsigned char * block;

    int first_block = start / 4;
    int offset;

    block = seq->sequence + first_block;
    offset = start % 4;
    
    i = 0;
    while (i < rsize) {
      seq_dest[i] = byte_to_base(*block, offset);

      ++i;
      ++offset;
      if (offset == 4) {
	offset = 0;
	++block;
      }
    }
    
  }

  /* fill in Ns */
  for (i = 0; i < seq->n_blocks; ++i) {
    uint32 bstart = seq->n_block_starts[i];
    uint32 bsize = seq->n_block_sizes[i];
    uint32 bend = bstart + bsize - 1;

    if (bstart <= end && bend >= start) {
      int j, k;

      if (bstart < start) {
	bsize -= (start - bstart);
	bstart = start;
      }

      for (j = 0, k = bstart; j < bsize && k <= end; ++j, ++k)
	seq_dest[k - start] = 'N';
    }
  }

  return result;
}

double * twobit_base_frequencies(TwoBit * ptr, const char * name) {
  struct twobit_index * seq = find_sequence(ptr->index, name);
  int size;
  double * result;
  int i;

  if (!seq) 
    return NULL;

  size = seq->size;
  result = (double*) calloc(4, sizeof(double));

  /* sum counts */
  {
    const unsigned char * block;

    int offset;

    block = seq->sequence;
    offset = 0;
    
    i = 0;
    while (i < size) {
      char base = byte_to_base(*block, offset);
      
      switch (base) { /* remap alphabet */
      case 'A': ++result[0]; break;
      case 'C': ++result[1]; break;
      case 'G': ++result[2]; break;
      case 'T': ++result[3]; break;
      }

      ++i;
      ++offset;
      if (offset == 4) {
	offset = 0;
	++block;
      }
    }
    
  }

  /* turn counts to frequencies */
  for (i = 0; i < 4; ++i)
    result[i] = result[i] / size;

  return result;
}

char ** twobit_sequence_names(TwoBit * ptr) {
  char ** result;
  int n_sequences = 0;
  struct twobit_index * idx;
  int i;

  for (idx = ptr->index; idx != NULL; idx = idx->next)
    ++n_sequences;

  result = (char**) calloc(n_sequences + 1, sizeof(char*));
  
  for (i = 0, idx = ptr->index; idx != NULL; ++i, idx = idx->next) {
    char * name = strdup(idx->name);
    result[i] = name;
  }

  return result;
}
