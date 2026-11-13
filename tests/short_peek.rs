use bincode_next::de::read::Reader;
use bincode_next::error::DecodeError;

struct ShortPeek {
    buf: Vec<u8>,
    pos: usize,
}

impl Reader for ShortPeek {
    fn read(
        &mut self,
        bytes: &mut [u8],
    ) -> Result<(), DecodeError> {
        let end = self.pos + bytes.len();
        if end > self.buf.len() {
            return Err(DecodeError::UnexpectedEnd {
                additional: end - self.buf.len(),
            });
        }
        bytes.copy_from_slice(&self.buf[self.pos..end]);
        self.pos = end;
        Ok(())
    }

    fn peek_read(
        &mut self,
        n: usize,
    ) -> Option<&[u8]> {
        let end = (self.pos + n).min(self.buf.len());
        Some(&self.buf[self.pos..end])
    }

    fn consume(
        &mut self,
        n: usize,
    ) {
        self.pos = (self.pos + n).min(self.buf.len());
    }
}

struct EmptyPeek;
impl Reader for EmptyPeek {
    fn read(
        &mut self,
        bytes: &mut [u8],
    ) -> Result<(), DecodeError> {
        Err(DecodeError::UnexpectedEnd {
            additional: bytes.len(),
        })
    }

    fn peek_read(
        &mut self,
        _n: usize,
    ) -> Option<&[u8]> {
        Some(&[])
    }
}

#[test]
fn u64_marker_only() {
    let r = ShortPeek {
        buf: vec![253u8],
        pos: 0,
    };
    let res: Result<u64, _> = bincode_next::decode_from_reader(r, bincode_next::config::standard());
    assert!(res.is_err());
}
#[test]
fn empty_peek() {
    let res: Result<u32, _> =
        bincode_next::decode_from_reader(EmptyPeek, bincode_next::config::standard());
    assert!(res.is_err());
}
#[test]
fn vec_len_prefix() {
    let r = ShortPeek {
        buf: vec![253u8],
        pos: 0,
    };
    let res: Result<Vec<u32>, _> =
        bincode_next::decode_from_reader(r, bincode_next::config::standard());
    assert!(res.is_err());
}
