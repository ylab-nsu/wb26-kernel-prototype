
pub enum RingBufferReadError {
    Empty,
}

pub enum RingBufferWriteError {
    Overflow,
}

pub struct RingBuffer<const N: usize> {
    buffer: [u8; N],
    head: usize,
    tail: usize,
    len: usize,
}

impl<const N: usize> RingBuffer<N> {
    pub const fn new() -> Self {
        RingBuffer {
            buffer: [0; N],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    pub fn write(&mut self, byte: u8) -> Result<(), RingBufferWriteError> {
        if self.len == N {
            Err(RingBufferWriteError::Overflow)
        } else {
            self.buffer[self.head] = byte;

            self.head = (self.head + 1) % N;
            self.len += 1;
            Ok(())
        }
    }

    pub fn read(&mut self) -> Result<u8, RingBufferReadError> {
        if self.len == 0 {
            Err(RingBufferReadError::Empty)
        } else {
            let byte = self.buffer[self.tail];

            self.tail = (self.tail + 1) % N;
            self.len -= 1;
            Ok(byte)
        }
    }

	pub fn is_empty(&self) -> bool {
    	self.len == 0
	}
}