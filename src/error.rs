//! Code for emitting and handling errors
//!
//! @DESIGN The goal is that if an error occurs we continue parsing the rest of the files
//! but I'm stil not sure whether copying should continue, stop or a rollback should occur.

struct Error {}
