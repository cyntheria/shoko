#[cfg(test)]
mod tests {
    use crate::archive::ShokoArchive;
    use std::fs;

    #[test]
    fn test_create_and_write_direct() {
        let test_path = "test_archive.sk1";
        let _ = fs::remove_file(test_path);
        let mut archive = ShokoArchive::create(test_path).unwrap();
        let content = b"wsg shoko heres some repeats or shi: AAAAAAAAAAAAAAAAAAAAA";
        archive.write_file_direct("test.txt", content, 5).unwrap();
        let mut reopened = ShokoArchive::open(test_path).unwrap();
        let extracted = reopened.extract_file("test.txt").unwrap();
        assert_eq!(content.to_vec(), extracted);
        fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_multi_file_append() {
        let test_path = "multi_test.sk1";
        let _ = fs::remove_file(test_path);
        let mut archive = ShokoArchive::create(test_path).unwrap();
        archive.write_file_direct("file1.bin", &[1, 2, 3], 0).unwrap();
        archive.write_file_direct("file2.bin", &[4, 5, 6], 9).unwrap();
        let mut reopened = ShokoArchive::open(test_path).unwrap();
        assert_eq!(reopened.entries.len(), 2);
        let f1 = reopened.extract_file("file1.bin").unwrap();
        let f2 = reopened.extract_file("file2.bin").unwrap();
        assert_eq!(f1, vec![1, 2, 3]);
        assert_eq!(f2, vec![4, 5, 6]);
        fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_deletion_and_defrag() {
        let test_path = "delete_test.sk1";
        let _ = fs::remove_file(test_path);
        let mut archive = ShokoArchive::create(test_path).unwrap();
        archive.write_file_direct("file1.txt", b"some data", 0).unwrap();
        archive.write_file_direct("file2.txt", b"more data here", 0).unwrap();
        let size_full = fs::metadata(test_path).unwrap().len();
        archive.delete_file("file1.txt").unwrap();
        assert_eq!(archive.entries.len(), 1);
        let reopened = ShokoArchive::open(test_path).unwrap();
        assert_eq!(reopened.entries.len(), 1);
        assert_eq!(reopened.entries[0].path, "file2.txt");
        archive.defrag().unwrap();
        let size_defragged = fs::metadata(test_path).unwrap().len();
        
        assert!(size_defragged < size_full, "Archive should be smaller after deleting a file and defragging");
        let content = archive.extract_file("file2.txt").unwrap();
        assert_eq!(content, b"more data here");
        fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_glob_matching() {
        let test_path = "glob_test.sk1";
        let _ = fs::remove_file(test_path);
        let mut archive = ShokoArchive::create(test_path).unwrap();
        archive.write_file_direct("logs/today.log", b"test", 0).unwrap();
        archive.write_file_direct("logs/yesterday.log", b"test", 0).unwrap();
        archive.write_file_direct("data/db.sqlite", b"test", 0).unwrap();
        archive.write_file_direct("README.md", b"test", 0).unwrap();
        let log_matches = archive.match_glob("logs/*.log").unwrap();
        assert_eq!(log_matches.len(), 2);
        assert!(log_matches.contains(&"logs/today.log".to_string()));
        let md_matches = archive.match_glob("*.md").unwrap();
        assert_eq!(md_matches.len(), 1);
        assert_eq!(md_matches[0], "README.md");
        fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_overwrite_integrity() {
        let test_path = "overwrite_integrity.sk1";
        let _ = fs::remove_file(test_path);
        let mut archive = ShokoArchive::create(test_path).unwrap();
        archive.write_file_direct("config.toml", b"key = value", 0).unwrap();
        archive.write_file_direct("config.toml", b"new_key = long_value_string", 0).unwrap();
        let mut reopened = ShokoArchive::open(test_path).unwrap();
        let data = reopened.extract_file("config.toml").unwrap();
        assert_eq!(data, b"new_key = long_value_string");
        assert_eq!(reopened.entries.len(), 1);
        fs::remove_file(test_path).unwrap();
    }
}
