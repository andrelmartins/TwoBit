#
# R twoBit Interface
#
#

#' Load a twoBit file
#'
#' @param filename character string with path to file
#' @return object representing a twoBit file
#' @useDynLib twobit rtwobit_load rtwobit_unload rtwobit_sequence rtwobit_sequence_freqs
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


#' Base frequencies per region
#'
#' @param twobit object representing a twoBit file
#' @param bed data frame with at least 'chrom', 'start' and 'end'
#' @return matrix of 4xN with frequencies for each of the N regions
#' @export
twobit.bed.frequencies <- function(twobit, bed) {
  N = dim(bed)[1]

  result = matrix(data=0, nrow=4, ncol=N)
  chroms = as.character(bed[,1])
  starts = as.integer(bed[,2])
  ends = as.integer(bed[,3])

  for (i in 1:N) {
    freqs = .Call(rtwobit_sequence_freqs, twobit, chroms[i], starts[i], ends[i])
    result[,i] = freqs
  }

  return(result)
}
