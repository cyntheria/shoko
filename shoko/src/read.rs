use std::io::{self, Read, Seek, SeekFrom};
use std::fs::File;
use crate::decompress::decompress;
use crate::encrypt;

pub struct ShokoReader<'a> {
    handle: &'a mut File,
}

impl<'a> ShokoReader<'a> {
    pub fn new(handle: &'a mut File) -> Self {
        Self { handle }
    }

    pub fn read_blob(&mut self, offset: u64, size: u64, clevel: u8) -> io::Result<Vec<u8>> {
        self.handle.seek(SeekFrom::Start(offset))?;
        
        let mut buffer = vec![0u8; size as usize];
        self.handle.read_exact(&mut buffer)?;

        let decrypted_buffer = encrypt::decrypt_data(&buffer)?;

        if clevel > 0 {
            decompress(&decrypted_buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        } else {
            Ok(decrypted_buffer)
        }
    }

    pub fn read_index_entry(&mut self) -> io::Result<(String, u64, u64, u8)> {
        let mut len_buf = [0u8; 4];
        self.handle.read_exact(&mut len_buf)?;
        let path_len = u32::from_le_bytes(len_buf) as usize;

        let mut path_bytes = vec![0u8; path_len];
        self.handle.read_exact(&mut path_bytes)?;
        let path = String::from_utf8_lossy(&path_bytes).into_owned();

        let mut size_buf = [0u8; 8];
        self.handle.read_exact(&mut size_buf)?;
        let size = u64::from_le_bytes(size_buf);

        let mut offset_buf = [0u8; 8];
        self.handle.read_exact(&mut offset_buf)?;
        let offset = u64::from_le_bytes(offset_buf);

        let mut clevel_buf = [0u8; 1];
        self.handle.read_exact(&mut clevel_buf)?;
        let clevel = clevel_buf[0];

        Ok((path, size, offset, clevel))
    }

    pub fn get_footer_info(&mut self) -> io::Result<(u64, u32)> {
        let file_len = self.handle.metadata()?.len();
        if file_len < 14 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "File too small"));
        }

        self.handle.seek(SeekFrom::End(-14))?;
        
        let mut footer_pos_buf = [0u8; 8];
        self.handle.read_exact(&mut footer_pos_buf)?;
        let index_start = u64::from_le_bytes(footer_pos_buf);

        let mut count_buf = [0u8; 4];
        self.handle.read_exact(&mut count_buf)?;
        let entry_count = u32::from_le_bytes(count_buf);

        let mut magic = [0u8; 2];
        self.handle.read_exact(&mut magic)?;
        if &magic != b"SK" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Shoko trailer"));
        }

        Ok((index_start, entry_count))
    }
}
