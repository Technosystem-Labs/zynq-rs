pub trait BytesTransferExt: Sized {
    // Turn u32 into u8
    fn bytes_transfer(self) -> BytesTransfer<Self>
    where
        Self: Iterator<Item = u32>;
}

impl<I: Iterator<Item = u32>> BytesTransferExt for I {
    // Turn u32 into u8
    fn bytes_transfer(self) -> BytesTransfer<Self> {
        BytesTransfer {
            iter: self,
            shift: 0,
            word: 0,
        }
    }
}

pub struct BytesTransfer<I: Iterator<Item = u32> + Sized> {
    iter: I,
    shift: u8,
    word: u32,
}

impl<I: Iterator<Item = u32> + Sized> Iterator for BytesTransfer<I> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.shift > 0 {
            self.shift -= 8;
            Some((self.word >> self.shift) as u8)
        } else {
            self.iter.next()
                .and_then(|word| {
                    self.shift = 32;
                    self.word = word;
                    self.next()
                })
        }
    }
}
