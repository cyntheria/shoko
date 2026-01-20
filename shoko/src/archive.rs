use std::fs::{File, OpenOptions};
use std::io::{self, Write, Seek, SeekFrom};
use crate::read::ShokoReader;
use crate::write::ShokoWriter;

pub struct ShokoEntry {
    pub path: String,
    pub size: u64,
    pub offset: u64,
    pub compression_level: u8,
}

pub struct ShokoArchive {
    pub(crate) file: File,
    pub entries: Vec<ShokoEntry>,
}

impl ShokoArchive {
    pub fn create(path: &str) -> io::Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        file.write_all(b"SHOKO001")?;
        
        Ok(Self {
            file,
            entries: Vec::new(),
        })
    }

    pub fn open(path: &str) -> io::Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;

        let mut entries = Vec::new();
        
        let footer_data = {
            let mut reader = ShokoReader::new(&mut file);
            reader.get_footer_info().ok()
        };

        if let Some((index_start, entry_count)) = footer_data {
            file.seek(SeekFrom::Start(index_start))?;
            let mut reader = ShokoReader::new(&mut file);
            for _ in 0..entry_count {
                let (path, size, offset, clevel) = reader.read_index_entry()?;
                entries.push(ShokoEntry {
                    path,
                    size,
                    offset,
                    compression_level: clevel,
                });
            }
        }

        Ok(Self { file, entries })
    }

    pub(crate) fn rewrite_index(&mut self) -> io::Result<()> {
        let index_start = if self.entries.is_empty() {
            8 
        } else {
            self.entries.iter()
                .map(|e| e.offset + e.size)
                .max()
                .unwrap_or(8)
        };

        self.file.seek(SeekFrom::Start(index_start))?;
        
        let mut writer = ShokoWriter::new(&mut self.file);
        for entry in &self.entries {
            writer.write_index_entry(
                &entry.path,
                entry.size,
                entry.offset,
                entry.compression_level,
            )?;
        }

        let entry_count = self.entries.len() as u32;
        writer.finalize(index_start, entry_count)?;
        
        let final_size = self.file.stream_position()?;
        self.file.set_len(final_size)?;
        Ok(())
    }

    pub fn write_file_direct(&mut self, internal_path: &str, content: &[u8], clevel: u8) -> io::Result<()> {
        let data_offset = if self.entries.is_empty() {
            8
        } else {
            self.entries.iter()
                .map(|e| e.offset + e.size)
                .max()
                .unwrap_or(8)
        };

        self.file.seek(SeekFrom::Start(data_offset))?;
        
        let compressed_size = {
            let mut writer = ShokoWriter::new(&mut self.file);
            writer.write_blob(content, clevel)?
        };

        self.entries.retain(|e| e.path != internal_path);
        self.entries.push(ShokoEntry {
            path: internal_path.to_string(),
            size: compressed_size,
            offset: data_offset,
            compression_level: clevel,
        });

        self.rewrite_index()
    }

    pub fn extract_file(&mut self, internal_path: &str) -> io::Result<Vec<u8>> {
        let entry = self.entries.iter()
            .find(|e| e.path == internal_path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File not in archive"))?;

        let mut reader = ShokoReader::new(&mut self.file);
        reader.read_blob(entry.offset, entry.size, entry.compression_level)
    }

    pub fn defrag(&mut self) -> io::Result<()> {
        let temp_path = ".shoko_defrag.tmp"; // this is kind of lazy to do ill refactor this later
        
        {
            let mut new_archive = ShokoArchive::create(temp_path)?;
            let paths: Vec<String> = self.entries.iter().map(|e| e.path.clone()).collect();

            for path in paths {
                let clevel = self.entries.iter()
                    .find(|e| e.path == path)
                    .map(|e| e.compression_level)
                    .unwrap_or(0);
                
                let data = self.extract_file(&path)?;
                new_archive.write_file_direct(&path, &data, clevel)?;
            }
        }

        let mut temp_file = File::open(temp_path)?;
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;
        io::copy(&mut temp_file, &mut self.file)?;
        let _ = std::fs::remove_file(temp_path);
        
        let mut reader = ShokoReader::new(&mut self.file);
        if let Ok((index_start, entry_count)) = reader.get_footer_info() {
            self.file.seek(SeekFrom::Start(index_start))?;
            self.entries.clear();
            let mut reader_inner = ShokoReader::new(&mut self.file);
            for _ in 0..entry_count {
                let (path, size, offset, clevel) = reader_inner.read_index_entry()?;
                self.entries.push(ShokoEntry { path, size, offset, compression_level: clevel });
            }
        }

        Ok(())
    }
}
