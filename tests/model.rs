#[cfg_attr(feature = "std", macro_use)]
extern crate quickcheck;
extern crate positioned_io2;
extern crate rand;
extern crate tempfile;
#[cfg(feature = "std")]
extern crate acid_io;

#[cfg(feature = "std")]
use std::cmp::{max, min};
#[cfg(feature = "std")]
use acid_io::{Read, Seek, SeekFrom, Write};

use self::quickcheck::{Arbitrary, Gen};
use self::rand::Rng;
#[cfg(feature = "std")]
use positioned_io2::{ReadAt, WriteAt};

#[derive(Clone, Debug)]
enum Op {
    WriteAll(Vec<u8>),
    WriteAllAt(u64, Vec<u8>),
    ReadExact(usize),
    ReadExactAt(u64, usize),
    Seek(u64),
    Flush,
}

impl Arbitrary for Op {
    fn arbitrary<G: Gen>(g: &mut G) -> Op {
        match g.gen_range(0, 6) {
            0 => Op::WriteAll(Vec::arbitrary(g)),
            1 => Op::WriteAllAt(g.gen_range(0, 123456), Vec::arbitrary(g)),
            2 => Op::ReadExact(usize::arbitrary(g)),
            3 => Op::ReadExactAt(u64::arbitrary(g), usize::arbitrary(g)),
            4 => Op::Seek(g.gen_range(0, 123456)),
            _ => Op::Flush,
        }
    }
}

#[cfg(feature = "std")]
struct Model {
    vec: Vec<u8>,
    pos: usize,
}

#[cfg(feature = "std")]
impl Model {
    fn new() -> Model {
        Model {
            vec: Vec::new(),
            pos: 0,
        }
    }

    fn write_all(&mut self, buf: &[u8]) {
        let len = max(self.vec.len(), self.pos + buf.len());
        self.vec.resize(len, 0);
        self.vec[self.pos..(self.pos + buf.len())].copy_from_slice(&buf);
        self.pos += buf.len();
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> bool {
        if buf.is_empty() {
            return true;
        }

        let len = buf.len();

        if self.pos + len > self.vec.len() {
            self.pos = self.vec.len();
            return false;
        }

        buf.copy_from_slice(&self.vec[self.pos..(self.pos + len)]);
        self.pos += len;
        true
    }
}

#[cfg(feature = "std")]
quickcheck! {
    fn file_matches_model(ops: Vec<Op>) -> bool {
        let mut file = tempfile::tempfile().unwrap();
        let mut model = Model::new();

        for op in ops {
            match op {
                Op::WriteAll(ref buf) => {
                    model.write_all(buf);
                    file.write_all(buf).unwrap();
                },
                Op::WriteAllAt(at, ref buf) => {
                    assert_eq!(model.vec.write_all_at(at, buf).is_ok(), file.write_all_at(at, buf).is_ok());
                },
                Op::ReadExact(bytes) => {
                    let mut a = vec![0; bytes];
                    let mut b = vec![0; bytes];

                    let success = model.read_exact(&mut a);
                    assert_eq!(success, file.read_exact(&mut b).is_ok());

                    if success {
                        assert_eq!(a, b);
                    }
                },
                Op::ReadExactAt(at, bytes) => {
                    let mut a = vec![0; bytes];
                    let mut b = vec![0; bytes];

                    let success = model.vec.read_exact_at(at, &mut a).is_ok();
                    assert_eq!(success, file.read_exact_at(at, &mut b).is_ok());

                    if success {
                        assert_eq!(a, b);
                    }
                }
                Op::Seek(pos) => {
                    // seeking past eof is implementation defined, so avoid that
                    model.pos = min(pos as usize, model.vec.len());
                    assert!(file.seek(SeekFrom::Start(model.pos as u64)).is_ok());
                }
                Op::Flush => {
                    Write::flush(&mut file).unwrap();
                }
            }
        }

        true
    }
}
