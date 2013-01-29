#
# R twoBit Interface
#
#

#' Load a twoBit file
#'
#' @param filename character string with path to file
#' @return object representing a twoBit file
#' @useDynLib twobit rtwobit_load rtwobit_unload rtwobit_sequence
#' @export
twobit.load <- function(filename) {
  out <- .Call(rtwobit_load, filename)
  return(out)
}

#' Unload 2bit file
#'
#' Frees C-side resources used to represent twoBit file
#' @param twobit object representing a twoBit file
#' @export
twobit.unload <- function(twobit) {
  stopifnot(!is.null(twobit))
  .Call(rtwobit_unload, twobit)
  invisible(NULL)
}

#' Extract the requested sequence from a twoBit file
#'
#' @param twobit object representing a twoBit file
#' @param name sequence name (chromosome)
#' @param start start coordinate (zero-based)
#' @param end end coordinate (zero-based)
#' @return character vector representing the requested string
#' @export
twobit.sequence <- function(twobit, name, start, end) {
  stopifnot(!is.null(twobit), !is.null(name), !is.null(start), !is.null(end))
  stopifnot(end >= start)
  out <- .Call(rtwobit_sequence, twobit, name, start, end)
  return(out)
}

#' Reverse complement sequence
#'
#' @param sequence DNA character string
#' @return character string
#' @export
twobit.reverse.complement <- function(sequence) {
  alphH = charToRaw("ACGT")
  revAlphaH = charToRaw("TGCA")
  seq = toupper(sequence)
  rawSeq = charToRaw(seq)
  rawSeq = rev(rawSeq)
  res = vector(mode="raw", length=length(rawSeq))

  for (i in 1:4)
    res[rawSeq == alphH[i]] = revAlphaH[i]
  res[res == 0] = charToRaw("N")
  rawToChar(res)
}


#' Convert DNA character sequence to integer sequence
#'
#' @param sequence DNA character string
#' @param base integer values for each nucleotide are A:base, C:base+1, G:base+2, T:base+3, N:base+4
#' @return integer vector
#' @export
twobit.sequence.to.integer <- function(sequence, base=1) {
  alphH = charToRaw("ACGTN")
  seq = toupper(sequence)
  rawSeq = charToRaw(seq)
  
  sapply(rawSeq, function(raw) base + which(alphH == raw) - 1)
}

