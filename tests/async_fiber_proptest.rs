#[cfg(feature = "async-fiber")]
mod proptests {
    use bincode_next::{config, decode_async, encode_to_vec, Decode, Encode};
    use proptest::prelude::*;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::AsyncRead;

    #[derive(Encode, Decode, PartialEq, Debug, Clone)]
    struct TestData {
        a: u64,
        b: String,
        c: Vec<u8>,
    }

    struct RandomChunkReader {
        data: Vec<u8>,
        chunk_sizes: Vec<usize>,
        chunk_idx: usize,
        pos: usize,
    }

    impl AsyncRead for RandomChunkReader {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            if self.pos >= self.data.len() {
                return Poll::Ready(Ok(()));
            }
            let chunk_size = *self.chunk_sizes.get(self.chunk_idx).unwrap_or(&1);
            self.chunk_idx += 1;
            let end = std::cmp::min(self.pos + chunk_size, self.data.len());
            buf.put_slice(&self.data[self.pos..end]);
            self.pos = end;
            Poll::Ready(Ok(()))
        }
    }

    proptest! {
        #[test]
        #[cfg_attr(miri, ignore)] 
        fn test_async_fiber_decode_random_chunks(
            chunk_sizes in prop::collection::vec(1..50usize, 100),
            a in any::<u64>(),
            b in ".*",
            c in prop::collection::vec(any::<u8>(), 0..100),
        ) {
            let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
            let original = TestData { a, b, c };
            let encoded = encode_to_vec(&original, config::standard()).unwrap();

            rt.block_on(async {
                let reader = RandomChunkReader {
                    data: encoded,
                    chunk_sizes,
                    chunk_idx: 0,
                    pos: 0,
                };
                let decoded: TestData = decode_async::<TestData, _, _>(config::standard(), reader).await.unwrap();
                prop_assert_eq!(original, decoded);
                Ok(())
            }).unwrap();
        }
    }
}
