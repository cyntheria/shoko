use std::io;
use crate::archive::ShokoArchive;

impl ShokoArchive {
    /// removes a file from the archive index, (well, duh why did i make a comment for this)
    /// note that this does not immediately reclaim disk space so call defrag() to optimize
    pub fn delete_file(&mut self, internal_path: &str) -> io::Result<()> {
        let original_len = self.entries.len();
        self.entries.retain(|e| e.path != internal_path);

        if self.entries.len() == original_len {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File '{}' not found in archive", internal_path),
            ));
        }
        self.rewrite_index()
    }
}
