<!-- cargo-rdme start -->

# skuld

Error and logging utility crate
Includes the following:
 - `bail`: A macro to return an error from a function
 - `location`: A macro to get the location of the call (i.e. a tuple with (file!(), line!(),
   column!())
 - `SkuldLogger`: A `log` crate facade that writes to the disk.

<!-- cargo-rdme end -->
