use std::{fmt::Write as FmtWrite, fs::File, io::Write};

pub struct BufferWriter {
    file: File,
    buffer: Vec<u8>,
}

impl BufferWriter {
    pub fn new(file: File) -> Self {
        // 4MB buffer
        let buffer = Vec::<u8>::with_capacity(4 * 2 ^ 20);
        BufferWriter { file, buffer }
    }

    pub fn flush(&mut self) {
        let _ = self.file.write_all(self.buffer.as_slice());

        self.buffer.clear();
    }
}

impl FmtWrite for BufferWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let bytes = s.as_bytes();

        if self.buffer.len() + bytes.len() >= self.buffer.capacity() {
            // This write will overflow buffer. Flush
            self.flush();
        }

        self.buffer.extend_from_slice(bytes);

        Ok(())
    }
}
