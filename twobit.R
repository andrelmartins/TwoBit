#
# R twoBit Interface
#

twobit.curPath = getwd()

# look for twobit.so
if (file.access(paste(twobit.curPath, "/twobit.so", sep='')) < 0) {
 # try hack
 twobit.curPath = dirname(sys.frame(1)$ofile)
 # test to see if it worked
 stopifnot(file.access(paste(twobit.curPath, "/twobit.so", sep='')) >= 0)
}

twobit.load <- function(filename) {
  dyn.load(paste(twobit.curPath, "/twobit.so", sep=''))
  out <- .Call("rtwobit_load", filename)
  return(out)
}

twobit.unload <- function(twobit) {
  stopifnot(!is.null(twobit))
  .Call("rtwobit_unload", twobit)
  invisible(NULL)
}

twobit.sequence <- function(twobit, name, start, end) {
  stopifnot(!is.null(twobit), !is.null(name), !is.null(start), !is.null(end))
  stopifnot(end >= start)
  out <- .Call("rtwobit_sequence", twobit, name, start, end)
  return(out)
}
