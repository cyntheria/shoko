use glob::Pattern; // im too lazy to implement glob pattern stuff from scratch, maybe in shoko2
use crate::archive::ShokoArchive;

impl ShokoArchive {
    pub fn match_glob(&self, pattern_str: &str) -> Result<Vec<String>, glob::PatternError> {
        let pattern = Pattern::new(pattern_str)?;
        let matches = self.entries.iter()
            .filter(|entry| pattern.matches(&entry.path))
            .map(|entry| entry.path.clone())
            .collect();
        Ok(matches)
    }
}
