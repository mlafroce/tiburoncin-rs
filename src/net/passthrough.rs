use crate::net::circular_buffer::CircularBuffer;
use crate::net::data_output::DataOutput;
use crate::net::BUFFER_SIZE;
use mio::net::TcpStream;
use mio::{Events, Interest, Poll};
use std::io;
use std::io::{Read, Write};

pub struct Passtrough {
    left: TcpStream,
    right: TcpStream,
    output: DataOutput,
}

const LEFT_CHANNEL: mio::Token = mio::Token(0);
const RIGHT_CHANNEL: mio::Token = mio::Token(1);

impl Passtrough {
    pub fn new(left: TcpStream, right: TcpStream) -> Self {
        Self {
            left,
            right,
            output: DataOutput::new(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(128);
        let mut event_buffer = [0u8; BUFFER_SIZE];
        let mut left_buffer: CircularBuffer<BUFFER_SIZE> = CircularBuffer::new();
        let mut right_buffer: CircularBuffer<BUFFER_SIZE> = CircularBuffer::new();

        poll.registry().register(
            &mut self.left,
            LEFT_CHANNEL,
            Interest::READABLE | Interest::WRITABLE,
        )?;
        poll.registry().register(
            &mut self.right,
            RIGHT_CHANNEL,
            Interest::READABLE | Interest::WRITABLE,
        )?;

        let mut left_writable = true;
        let mut right_writable = true;

        loop {
            poll.poll(&mut events, None)?;

            for event in events.iter() {
                match event.token() {
                    LEFT_CHANNEL => {
                        if event.is_readable() {
                            while let Ok(read_bytes) = Passtrough::read_channel(
                                &mut self.left,
                                &mut left_buffer,
                                &mut event_buffer,
                            ) {
                                if read_bytes == 0 {
                                    break;
                                };
                                self.output
                                    .print_read_bytes(&event_buffer, read_bytes, true);
                                self.output.print_remaining_bytes(true, &left_buffer);
                                if right_writable {
                                    Passtrough::write_channel(
                                        &mut self.right,
                                        &mut left_buffer,
                                        &mut event_buffer,
                                        &mut self.output,
                                    )?;
                                }
                                self.output.print_remaining_bytes(true, &left_buffer);
                            }
                            right_writable = false;
                        }
                        if event.is_writable() {
                            if left_buffer.ready_bytes() != 0 {
                                Passtrough::write_channel(
                                    &mut self.right,
                                    &mut left_buffer,
                                    &mut event_buffer,
                                    &mut self.output,
                                )?;
                                self.output.print_remaining_bytes(true, &left_buffer);
                            } else {
                                left_writable = true;
                            }
                        }
                    }
                    RIGHT_CHANNEL => {
                        if event.is_readable() {
                            while let Ok(read_bytes) = Passtrough::read_channel(
                                &mut self.right,
                                &mut right_buffer,
                                &mut event_buffer,
                            ) {
                                if read_bytes == 0 {
                                    break;
                                };
                                self.output
                                    .print_read_bytes(&event_buffer, read_bytes, false);
                                self.output.print_remaining_bytes(false, &right_buffer);
                                if left_writable {
                                    Passtrough::write_channel(
                                        &mut self.left,
                                        &mut right_buffer,
                                        &mut event_buffer,
                                        &mut self.output,
                                    )?;
                                }
                                self.output.print_remaining_bytes(false, &right_buffer);
                            }
                            left_writable = false;
                        }
                        if event.is_writable() {
                            if right_buffer.ready_bytes() != 0 {
                                Passtrough::write_channel(
                                    &mut self.left,
                                    &mut right_buffer,
                                    &mut event_buffer,
                                    &mut self.output,
                                )?;
                                self.output.print_remaining_bytes(false, &right_buffer);
                            } else {
                                right_writable = true;
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn read_channel(
        read_stream: &mut TcpStream,
        stream_buffer: &mut CircularBuffer<BUFFER_SIZE>,
        event_buffer: &mut [u8],
    ) -> io::Result<usize> {
        let mut buffer_slice = event_buffer.get_mut(0..stream_buffer.free_bytes()).unwrap();
        let read_bytes = read_stream.read(&mut buffer_slice)?;
        if let Some(buffer_content) = event_buffer.get(0..read_bytes) {
            stream_buffer.buffer_read(buffer_content);
        }
        Ok(read_bytes)
    }

    fn write_channel(
        write_stream: &mut TcpStream,
        stream_buffer: &mut CircularBuffer<BUFFER_SIZE>,
        event_buffer: &mut [u8],
        _output: &mut DataOutput,
    ) -> io::Result<usize> {
        let buffer_slice = event_buffer
            .get_mut(0..stream_buffer.ready_bytes())
            .unwrap();
        stream_buffer.buffer_write(buffer_slice);
        let write_bytes = write_stream.write(&buffer_slice)?;
        stream_buffer.advance_tail(write_bytes);
        Ok(write_bytes)
    }
}
