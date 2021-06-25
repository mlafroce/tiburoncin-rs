#[derive(Debug)]
/// Circular static buffer for socket data
pub struct CircularBuffer<const SIZE: usize> {
    buffer: [u8; SIZE],
    head: usize,
    tail: usize,
    buffer_full: bool,
}

impl<const SIZE: usize> CircularBuffer<SIZE> {
    /// Create an empty circular buffer of size `SIZE`
    pub fn new() -> Self {
        let buffer = [0u8; SIZE];
        let head = 0;
        let tail = 0;
        let buffer_full = false;
        CircularBuffer {
            buffer,
            head,
            tail,
            buffer_full,
        }
    }

    /// Returns non used bytes
    pub fn free_bytes(&self) -> usize {
        if self.buffer_full {
            return 0;
        }
        if self.head < self.tail {
            self.tail - self.head
        } else {
            SIZE - self.head + self.tail
        }
    }

    /// Return used bytes that can be read
    pub fn ready_bytes(&self) -> usize {
        if self.buffer_full {
            return SIZE;
        }
        if self.head < self.tail {
            SIZE - self.tail + self.head
        } else {
            self.head - self.tail
        }
    }

    /// Reads N bytes where N is the size `buf` param
    /// contents of `buf` are copied into circular buffer
    /// `buf` cannot be larger than the circular buffer
    pub fn buffer_read(&mut self, buf: &[u8]) {
        let remaining = SIZE - self.head;
        if remaining < buf.len() {
            self.buffer[self.head..].copy_from_slice(&buf[..remaining]);
            self.head = buf.len() - remaining;
            self.buffer[0..self.head].copy_from_slice(&buf[remaining..]);
        } else {
            self.buffer[self.head..self.head + buf.len()].copy_from_slice(&buf);
            self.head += buf.len();
        }
        self.head %= SIZE;
        if self.head == self.tail {
            self.buffer_full = true;
        }
    }

    /// Writes N bytes where N is the size `dest` param
    /// contents of circular buffer are copied into `dest`
    /// `des` cannot be larger than the circular buffer
    pub fn buffer_write(&mut self, dest: &mut [u8]) {
        if self.tail < self.head {
            dest.copy_from_slice(&self.buffer[self.tail..self.head]);
        } else {
            let remaining = SIZE - self.tail;
            dest[..remaining].copy_from_slice(&self.buffer[self.tail..]);
            if self.head > 0 {
                dest[remaining..].copy_from_slice(&self.buffer[..self.head]);
            }
        }
    }

    /// After bytes where read, advance buffer tail.
    pub fn advance_tail(&mut self, offset: usize) {
        self.tail = (self.tail + offset) % SIZE;
        self.buffer_full = false;
    }
}

impl<const SIZE: usize> Default for CircularBuffer<SIZE> {
    fn default() -> Self {
        CircularBuffer::<SIZE>::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_read() {
        let src_buffer = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut dest_buffer = [0u8; 8];
        let mut circular_buffer: CircularBuffer<8> = CircularBuffer::new();
        for buf_size in 1..9 {
            let mut expected = [0u8; 8];
            for i in 0..buf_size {
                expected[i] = i as u8 + 1;
            }
            for _ in 0..100 {
                circular_buffer.buffer_read(&src_buffer[0..buf_size]);
                circular_buffer.buffer_write(&mut dest_buffer[0..buf_size]);
                circular_buffer.advance_tail(buf_size);
                assert_eq!(dest_buffer, expected);
            }
        }
    }

    #[test]
    fn read_pairs() {
        let mut circular_buffer: CircularBuffer<5> = CircularBuffer::new();
        let mut dest_buffer = [0u8; 5];
        circular_buffer.buffer_read(&[1, 1]);
        circular_buffer.buffer_read(&[2, 2]);
        let ready_bytes = circular_buffer.ready_bytes();
        assert_eq!(ready_bytes, 4);
        circular_buffer.buffer_write(&mut dest_buffer[0..ready_bytes]);
        circular_buffer.advance_tail(ready_bytes);
        assert_eq!(dest_buffer[0..ready_bytes], [1, 1, 2, 2]);

        circular_buffer.buffer_read(&[3, 3]);
        let ready_bytes = circular_buffer.ready_bytes();
        assert_eq!(ready_bytes, 2);
        circular_buffer.buffer_write(&mut dest_buffer[0..ready_bytes]);
        circular_buffer.advance_tail(ready_bytes);
        assert_eq!(dest_buffer[..ready_bytes], [3, 3]);

        circular_buffer.buffer_read(&[1, 2]);
        circular_buffer.buffer_read(&[3, 4, 5]);
        let ready_bytes = circular_buffer.ready_bytes();
        println!("{:?}", ready_bytes);
        circular_buffer.buffer_write(&mut dest_buffer[0..ready_bytes]);
        circular_buffer.advance_tail(ready_bytes);
        assert_eq!(dest_buffer[..ready_bytes], [1, 2, 3, 4, 5]);
    }
}
