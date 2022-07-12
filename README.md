positioned-io2
==============

This crate allows you to specify an offset for reads and writes, without changing the current
position in a file. This is similar to [`pread()` and `pwrite()`][pread] in C.

The major advantages of this type of I/O are:

* You don't need to seek before doing a random-access read or write, which is convenient.
* Reads don't modify the file at all, so don't require mutability.

[pread]: http://man7.org/linux/man-pages/man2/pread.2.html

[![Crates.io](https://img.shields.io/crates/v/positioned-io2.svg)](https://crates.io/crates/positioned-io2)
[![Documentation](https://docs.rs/positioned-io2/badge.svg)](https://docs.rs/positioned-io2)

Fork
----

This is a fork of [positioned-io](https://github.com/vasi/positioned-io), which seem to have become unmaintained.

Example
-------

Read the fifth 512-byte sector of a file:

```rust
use std::fs::File;
use positioned_io2::ReadAt;

// note that file does not need to be mut
let file = File::open("tests/pi.txt")?;

// read up to 512 bytes
let mut buf = [0; 512];
let bytes_read = file.read_at(2048, &mut buf)?;
```

**Note:** If possible use the `RandomAccessFile` wrapper. On Windows `ReadAt` directly on `File` is very slow.

License
-------

positioned-io2 is licensed under the [MIT license](https://github.com/surban/positioned-io2/blob/master/LICENSE-MIT).
