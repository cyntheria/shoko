use std::io::{self, Write, Seek};
use std::fs::File;
use crate::compress::compress;
use crate::encrypt;

pub struct ShokoWriter<'a> {
    handle: &'a mut File,
}

impl<'a> ShokoWriter<'a> {
    pub fn new(handle: &'a mut File) -> Self {
        Self { handle }
    }

    pub fn write_blob(&mut self, data: &[u8], clevel: u8) -> io::Result<u64> {
        let processed_data = if clevel > 0 {
            compress(data, clevel)
        } else {
            data.to_vec()
        };

        let encrypted_data = encrypt::encrypt_data(&processed_data)?;

        let start_pos = self.handle.stream_position()?;
        self.handle.write_all(&encrypted_data)?;
        let end_pos = self.handle.stream_position()?;

        Ok(end_pos - start_pos)
    }

    pub fn write_index_entry(&mut self, path: &str, size: u64, offset: u64, clevel: u8) -> io::Result<()> {
        let path_bytes = path.as_bytes();
        self.handle.write_all(&(path_bytes.len() as u32).to_le_bytes())?;
        self.handle.write_all(path_bytes)?;
        self.handle.write_all(&size.to_le_bytes())?;
        self.handle.write_all(&offset.to_le_bytes())?;
        self.handle.write_all(&[clevel])?;
        Ok(())
    }

    pub fn finalize(&mut self, index_start: u64, entry_count: u32) -> io::Result<()> {
        self.handle.write_all(&index_start.to_le_bytes())?;
        self.handle.write_all(&count_to_bytes(entry_count))?;
        self.handle.write_all(b"SK")?;
        Ok(())
    }
}

fn count_to_bytes(count: u32) -> [u8; 4] {
    count.to_le_bytes()
}
