use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct ShokoMetadata {
    pub mode: u32,             
    pub modified: u64,         
    pub created: u64,          
}

impl Default for ShokoMetadata {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            mode: 0o644,
            modified: now,
            created: now,
        }
    }
}
