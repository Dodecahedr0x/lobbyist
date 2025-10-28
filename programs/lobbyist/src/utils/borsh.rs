use borsh::io::{self, Read};

pub struct SliceReader<'a> {
    buf: &'a [u8],
}

impl<'a> SliceReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

impl<'a> Read for SliceReader<'a> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let n = out.len().min(self.buf.len());
        if n == 0 {
            return Ok(0);
        }
        let (head, tail) = self.buf.split_at(n);
        out[..n].copy_from_slice(head);
        self.buf = tail;
        Ok(n)
    }
}
