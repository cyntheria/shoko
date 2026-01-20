use std::io::{self, Write, Seek, SeekFrom};
use std::fs::File;
use crate::compress::compress;

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

        let start_pos = self.handle.stream_position()?;
        self.handle.write_all(&processed_data)?;
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
        self.handle.write_all(&entry_count.to_le_bytes())?;
        self.handle.write_all(b"SK")?;
        Ok(())
    }

    pub fn seek_to_end(&mut self) -> io::Result<u64> {
        self.handle.seek(SeekFrom::End(0))
    }
}
