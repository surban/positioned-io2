use core::cell::RefCell;
use acid_io::{Read, Seek, SeekFrom, Write};
use ::{ReadAt, WriteAt};
use Size;

/// A wrapper that implements [`ReadAt`] and [`WriteAt`] for types
/// implementing `Seek` and `Read` / `Write` respectively.
///
/// # Example
///
/// ```rust
/// # extern crate acid_io;
/// # use acid_io::{Cursor, Result};
/// use positioned_io2::ReadAt;
/// # fn try_main() -> Result<()> {
/// use positioned_io2::{SeekWrapper, ReadAt, WriteAt};
/// let mut vec = vec![0, 1, 2];
/// let mut cursor = Cursor::new(vec);
/// let mut wrapper = SeekWrapper::new(cursor);
/// wrapper.write_at(2, &[3, 4, 5])?;
/// let mut buf = [0; 3];
/// wrapper.read_exact_at(1, &mut buf)?;
/// assert_eq!(buf, [1, 3, 4]);
/// # Ok(())
/// # }
/// # try_main().unwrap();
/// ```
#[derive(Debug)]
pub struct SeekWrapper<T> {
    inner: RefCell<SeekWrapperInner<T>>,
}
#[derive(Debug)]
struct SeekWrapperInner<T> {
    position: Option<u64>,
    inner: T,
}


impl<T> SeekWrapper<T> {
    /// Creates a new wrapper around the inner reader / writer.
    pub const fn new(inner: T) -> SeekWrapper<T> {
        SeekWrapper { inner: RefCell::new(SeekWrapperInner { position: None, inner }) }
    }
}

impl<T: Read + Seek> ReadAt for SeekWrapper<T> {
    fn read_at(&self, pos: u64, buf: &mut [u8]) -> acid_io::Result<usize> {
        let mut inner = self.inner.borrow_mut();

        if inner.position != Some(pos) {
            inner.inner.seek(SeekFrom::Start(pos))?;
        }
        let read = inner.inner.read(buf)?;
        inner.position = Some(pos + read as u64);
        Ok(read)
    }

    fn read_exact_at(&self, pos: u64, buf: &mut [u8]) -> acid_io::Result<()> {
        let mut inner = self.inner.borrow_mut();

        if inner.position != Some(pos) {
            inner.inner.seek(SeekFrom::Start(pos))?;
        }
        inner.inner.read_exact(buf)?;
        inner.position = Some(pos + buf.len() as u64);
        Ok(())
    }
}

impl<T: Write + Seek> WriteAt for SeekWrapper<T> {
    fn write_at(&mut self, pos: u64, buf: &[u8]) -> acid_io::Result<usize> {
        let mut inner = self.inner.get_mut();

        if inner.position != Some(pos) {
            inner.inner.seek(SeekFrom::Start(pos))?;
        }
        let written = inner.inner.write(buf)?;
        inner.position = Some(pos + written as u64);
        Ok(written)
    }

    fn write_all_at(&mut self, pos: u64, buf: &[u8]) -> acid_io::Result<()> {
        let mut inner = self.inner.get_mut();

        if inner.position != Some(pos) {
            inner.inner.seek(SeekFrom::Start(pos))?;
        }
        inner.inner.write_all(buf)?;
        inner.position = Some(pos + buf.len() as u64);
        Ok(())
    }

    fn flush(&mut self) -> acid_io::Result<()> {
        self.inner.get_mut().inner.flush()
    }
}

impl<T: Seek> Size for SeekWrapper<T> {
    fn size(&self) -> acid_io::Result<Option<u64>> {
        let mut borrow = self.inner.borrow_mut();
        let pos = borrow.inner.seek(SeekFrom::End(0))?;
        borrow.position = Some(pos);
        Ok(Some(pos))
    }
}
