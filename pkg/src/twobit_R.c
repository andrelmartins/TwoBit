/* R interface for wiglib */
#include "twobit.h"
#include <R.h>
#include <Rdefines.h>

static void rtwobit_finalizer(SEXP ptr) {
  if (!R_ExternalPtrAddr(ptr)) return;
  twobit_close(R_ExternalPtrAddr(ptr));
  R_ClearExternalPtr(ptr); /* not really necessary */
}

static SEXP rtwobit_names(TwoBit * twobit) {
  char ** names = twobit_sequence_names(twobit);
  SEXP result;
  int count = 0, i;
  char ** ptr;

  for (ptr = names; *ptr != NULL; ++ptr)
    ++count;

  PROTECT(result = allocVector(STRSXP, count));

  for (ptr = names, i = 0; *ptr != NULL; ++ptr, ++i) {
    SET_STRING_ELT(result, i, mkChar(*ptr));
    free(*ptr);
  }
  free(names);

  UNPROTECT(1);

  return result;
}

/* Load / Unload */
SEXP rtwobit_load(SEXP filename) {
  SEXP ans, ans_names, ptr;
  TwoBit * twobit;


  /* load twobit file */
  PROTECT(filename = AS_CHARACTER(filename));

  twobit = twobit_open(CHAR(STRING_ELT(filename, 0)));

  /* create answer */
  if (twobit == NULL) {
    UNPROTECT(1);
    return R_NilValue;
  }

  PROTECT(ans = allocVector(VECSXP, 1)); /* names */
  PROTECT(ans_names = allocVector(STRSXP, 1));

  /* make external pointer */
  ptr = R_MakeExternalPtr(twobit, install("TWOBIT_struct"), R_NilValue);
  PROTECT(ptr);
  R_RegisterCFinalizerEx(ptr, rtwobit_finalizer, TRUE);
  setAttrib(ans, install("handle_ptr"), ptr);

  /* fill info */
  SET_VECTOR_ELT(ans, 0, rtwobit_names(twobit));
  SET_STRING_ELT(ans_names, 0, mkChar("names"));

  setAttrib(ans, R_NamesSymbol, ans_names);
  UNPROTECT(4);

  return ans;
}

void rtwobit_unload(SEXP obj) {
  SEXP ptr;

  PROTECT(ptr = GET_ATTR(obj, install("handle_ptr")));
  if (ptr == R_NilValue)
    error("invalid twobit object");

  rtwobit_finalizer(ptr);

  UNPROTECT(1);
}

SEXP rtwobit_sequence(SEXP obj, SEXP name, SEXP start, SEXP end) {
  SEXP ptr, res = R_NilValue;
  TwoBit * twobit;

  PROTECT(name = AS_CHARACTER(name));
  PROTECT(start = AS_INTEGER(start));
  PROTECT(end = AS_INTEGER(end));

  PROTECT(ptr = GET_ATTR(obj, install("handle_ptr")));
  if (ptr == R_NilValue)
    error("invalid twobit object");

  twobit = R_ExternalPtrAddr(ptr);
  if (twobit == NULL) {
    error("twobit object has been unloaded");
  } else {
    char * seq = twobit_sequence(twobit, CHAR(STRING_ELT(name, 0)),
				 INTEGER(start)[0], INTEGER(end)[0]);
    if (seq == NULL)
      error("unknown sequence or invalid range: %s:%d-%d", CHAR(STRING_ELT(name, 0)), INTEGER(start)[0], INTEGER(end)[0]);
    else {
      PROTECT(res = allocVector(STRSXP, 1));
      SET_STRING_ELT(res, 0, mkChar(seq));
      free(seq);
      UNPROTECT(1);
    }
  }

  UNPROTECT(4);

  return res;
}

SEXP rtwobit_sequence_freqs(SEXP obj, SEXP name, SEXP start, SEXP end) {
  SEXP ptr, res = R_NilValue;
  TwoBit * twobit;

  PROTECT(name = AS_CHARACTER(name));
  PROTECT(start = AS_INTEGER(start));
  PROTECT(end = AS_INTEGER(end));

  PROTECT(ptr = GET_ATTR(obj, install("handle_ptr")));
  if (ptr == R_NilValue)
    error("invalid twobit object");

  twobit = R_ExternalPtrAddr(ptr);
  if (twobit == NULL) {
    error("twobit object has been unloaded");
  } else {
    char * seq = twobit_sequence(twobit, CHAR(STRING_ELT(name, 0)),
				 INTEGER(start)[0], INTEGER(end)[0]);
    if (seq == NULL)
      error("unknown sequence or invalid range: %s:%d-%d", CHAR(STRING_ELT(name, 0)), INTEGER(start)[0], INTEGER(end)[0]);
    else {
      int cA = 0, cC = 0, cG = 0, cT = 0;
      double total;
      int i, len;
      double * ptr;

      PROTECT(res = NEW_NUMERIC(4));
      ptr = REAL(res);

      len = strlen(seq);

      for (i = 0; i < len; ++i) {
	switch (seq[i]) {
	case 'A': ++cA; break;
	case 'C': ++cC; break;
	case 'G': ++cG; break;
	case 'T': ++cT; break;
	}
      }

      total = cA + cC + cG + cT;
      ptr[0] = cA / total;
      ptr[1] = cC / total;
      ptr[2] = cG / total;
      ptr[3] = cT / total;

      free(seq);
      UNPROTECT(1);
    }
  }
  UNPROTECT(4);

  return res;
}
