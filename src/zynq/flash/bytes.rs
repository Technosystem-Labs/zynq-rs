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
            shift: 32,
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
        if self.shift < 24 {
            self.shift += 8;
            Some((self.word >> self.shift) as u8)
        } else {
            self.shift = 0;
            self.iter.next()
                .map(|word| {
                    self.word = word;
                    word as u8
                })
        }
    }
}
