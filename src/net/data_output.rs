use crate::net::circular_buffer::CircularBuffer;
use crate::net::BUFFER_SIZE;

pub struct DataOutput {
    left_read_bytes: usize,
    right_read_bytes: usize,
}

pub const LINE_WIDTH: usize = 16;

impl DataOutput {
    pub fn new() -> Self {
        Self {
            left_read_bytes: 0,
            right_read_bytes: 0,
        }
    }
}

impl DataOutput {
    /// Prints bytes read into the buffer, with source and destination
    pub fn print_read_bytes(&mut self, event_buffer: &[u8], read_bytes: usize, is_left: bool) {
        let (src, dest) = if is_left { ("A", "B") } else { ("B", "A") };
        println!("{} -> {} sent {} bytes", src, dest, read_bytes);
        let offset_bytes;
        if is_left {
            offset_bytes = self.left_read_bytes;
            self.left_read_bytes += read_bytes;
        } else {
            offset_bytes = self.right_read_bytes;
            self.right_read_bytes += read_bytes;
        }
        let mut printed_bytes = 0;
        while printed_bytes < read_bytes {
            let line_offset = offset_bytes + printed_bytes;
            let remaining_bytes = read_bytes - printed_bytes;
            printed_bytes += DataOutput::print_hex_line(
                offset_bytes + printed_bytes,
                event_buffer,
                printed_bytes,
                remaining_bytes,
                line_offset,
            );
        }
    }

    /// Prints remaining bytes in the buffer
    pub fn print_remaining_bytes(
        &self,
        is_left: bool,
        stream_buffer: &CircularBuffer<BUFFER_SIZE>,
    ) {
        let src = if is_left { "A" } else { "B" };
        let ready_bytes = stream_buffer.ready_bytes();
        if ready_bytes > 0 {
            println!("{} is {} bytes behind", src, ready_bytes);
        } else {
            println!("{} is synced", src);
        }
    }

    fn print_hex_line(
        offset_bytes: usize,
        event_buffer: &[u8],
        buffer_offset: usize,
        remaining_bytes: usize,
        line_offset: usize,
    ) -> usize {
        print!("{:08x} ", offset_bytes);
        let line_byte = line_offset % LINE_WIDTH;
        let mut printed = 0;
        if line_byte < LINE_WIDTH / 2 {
            printed =
                DataOutput::print_half_hex(event_buffer, buffer_offset, remaining_bytes, line_byte);
        } else {
            for _ in 0..LINE_WIDTH / 2 {
                print!("   ");
            }
        }
        print!(" ");
        let half_line_byte =
            std::cmp::max(line_offset % LINE_WIDTH, LINE_WIDTH / 2) - LINE_WIDTH / 2;
        printed += DataOutput::print_half_hex(
            event_buffer,
            buffer_offset + printed,
            remaining_bytes - printed,
            half_line_byte,
        );
        print!(" ");
        DataOutput::print_ascii(event_buffer, buffer_offset, line_byte, printed);
        println!("");

        printed
    }

    fn print_half_hex(
        event_buffer: &[u8],
        buffer_offset: usize,
        remaining_bytes: usize,
        half_line_byte: usize,
    ) -> usize {
        let mut cur_line_byte = half_line_byte;
        for _ in 0..half_line_byte {
            print!("   ");
        }
        for i in 0..remaining_bytes {
            let byte = event_buffer[buffer_offset + i];
            print!(" {:02x}", byte);
            if cur_line_byte == LINE_WIDTH / 2 - 1 {
                return i + 1;
            }
            cur_line_byte += 1;
        }
        for _ in cur_line_byte.. LINE_WIDTH / 2 {
            print!("   ");
        }
        cur_line_byte - half_line_byte
    }

    fn print_ascii(event_buffer: &[u8], buffer_offset: usize, line_offset: usize, remaining_bytes: usize) {
        print!("|");
        for _ in 0..line_offset {
            print!(" ");
        }
        for c in 0..remaining_bytes {
            let byte = event_buffer[buffer_offset + c];
            if DataOutput::printable(byte) {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }
        for _ in line_offset + remaining_bytes .. LINE_WIDTH {
            print!(" ");
        }
        print!("|");
    }

    fn printable(byte: u8) -> bool {
        u8::wrapping_sub(byte, 0x20) < 0x5f
    }
}

impl Default for DataOutput {
    fn default() -> Self {
        DataOutput::new()
    }
}
