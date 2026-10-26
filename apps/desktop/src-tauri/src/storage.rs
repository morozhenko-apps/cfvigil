//! Durable private JSON persistence. A malformed file is preserved under a
//! unique .corrupt.* sibling name; never silently destroy incident history.
//! All disk errors are returned without leaking filenames, tokens or JSON.
use crate::native_messages as msg;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub const MAX_JSON_BYTES: u64 = 4 * 1024 * 1024;

pub struct Loaded<T> {
    pub value: T,
    pub quarantined: bool,
}


fn unique_suffix() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)
        .unwrap_or_default().as_nanos();
    format!("{}.{}", std::process::id(), nanos)
}

fn refuse_link(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(info) if info.file_type().is_symlink() || !info.is_file() =>
            Err(msg::STORAGE_NOT_FILE.into()),
        Ok(_) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(msg::STORAGE_INSPECT.into()),
    }
}

fn quarantine(path: &Path) -> Result<(), String> {
    refuse_link(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|_| msg::STORAGE_PROTECT)?;
    }
    let name = path.file_name().and_then(|name| name.to_str())
        .ok_or(msg::STORAGE_FILENAME)?;
    let backup = path.with_file_name(format!("{name}.corrupt.{}", unique_suffix()));
    if backup.exists() {
        return Err(msg::STORAGE_BACKUP_EXISTS.into());
    }
    fs::rename(path, backup)
        .map_err(|_| msg::STORAGE_BACKUP_FAILED.into())
}

/// Missing file is normal on first use. Invalid/oversized content is moved
/// intact to quarantine before default data may be used and subsequently
/// saved. Permission and other I/O errors are never treated as empty data.
pub fn read_or_default<T: DeserializeOwned + Default>(
    path: &Path,
    max_size: u64,
) -> Result<Loaded<T>, String> {
    refuse_link(path)?;
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound =>
            return Ok(Loaded { value: T::default(), quarantined: false }),
        Err(_) => return Err(msg::STORAGE_OPEN.into()),
    };
    // Harden permissions on pre-existing valid JSON created by older
    // releases as well as every new file written by this storage module.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|_| msg::STORAGE_PERMISSIONS)?;
    }
    let mut bytes = Vec::new();
    (&mut file).take(max_size.saturating_add(1)).read_to_end(&mut bytes)
        .map_err(|_| msg::STORAGE_READ)?;
    if bytes.len() as u64 > max_size {
        drop(file);
        quarantine(path)?;
        eprintln!("CFVigil: oversized JSON was preserved in a damaged-state backup");
        return Ok(Loaded { value: T::default(), quarantined: true });
    }
    match serde_json::from_slice(&bytes) {
        Ok(value) => Ok(Loaded { value, quarantined: false }),
        Err(_) => {
            drop(file);
            quarantine(path)?;
            eprintln!("CFVigil: malformed JSON was preserved in a damaged-state backup");
            Ok(Loaded { value: T::default(), quarantined: true })
        }
    }
}

/// Each write uses a fresh private file in the target directory, flushes it,
/// and atomically swaps into place. Interrupted writes cannot truncate the
/// last complete JSON file. Secrets still belong only in the OS keychain.
pub fn write_atomic<T: Serialize>(path: &Path, value: &T, max_size: u64) -> Result<(), String> {
    refuse_link(path)?;
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|_| msg::STORAGE_SERIALIZE)?;
    if bytes.len() as u64 > max_size {
        return Err(msg::STORAGE_BIG.into());
    }
    let name = path.file_name().and_then(|name| name.to_str())
        .ok_or(msg::STORAGE_FILENAME)?;
    let temp = path.with_file_name(format!("{name}.tmp.{}", unique_suffix()));
    let result = (|| -> std::io::Result<()> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        // Replacement is atomic on the same filesystem, not a remove/write.
        fs::rename(&temp, path)?;
        #[cfg(unix)]
        if let Some(parent) = path.parent() {
            File::open(parent)?.sync_all()?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
        return Err(msg::STORAGE_PERSIST.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);

    struct TestDir(std::path::PathBuf);
    impl TestDir {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "cfvigil-store-{}-{}-{}", std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed), unique_suffix()));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
        fn file(&self) -> std::path::PathBuf { self.0.join("events.json") }
    }
    impl Drop for TestDir {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn atomic_write_preserves_existing_history_and_private_permissions() {
        let dir = TestDir::new();
        let path = dir.file();
        write_atomic(&path, &vec!["old"], MAX_JSON_BYTES).unwrap();
        assert_eq!(read_or_default::<Vec<String>>(&path, MAX_JSON_BYTES).unwrap().value, vec!["old"]);
        write_atomic(&path, &vec!["new", "event"], MAX_JSON_BYTES).unwrap();
        assert_eq!(read_or_default::<Vec<String>>(&path, MAX_JSON_BYTES).unwrap().value, vec!["new","event"]);
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        }
    }

    #[cfg(unix)]
    #[test]
    fn preexisting_valid_json_is_hardened_to_private_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TestDir::new();
        let path = dir.file();
        fs::write(&path, b"[\"private\"]").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        let result = read_or_default::<Vec<String>>(&path, MAX_JSON_BYTES).unwrap();
        assert!(!result.quarantined);
        assert_eq!(result.value, vec!["private"]);
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn corrupt_content_is_preserved_before_reset_and_rewrite() {
        let dir = TestDir::new();
        let path = dir.file();
        fs::write(&path, b"{truncated").unwrap();
        let loaded: Loaded<Vec<String>> = read_or_default(&path, MAX_JSON_BYTES).unwrap();
        assert!(loaded.value.is_empty());
        assert!(loaded.quarantined);
        assert!(!path.exists());
        let backups: Vec<_> = fs::read_dir(&dir.0).unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|p| p.file_name().unwrap().to_string_lossy().contains(".corrupt.")).collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), b"{truncated");
        write_atomic(&path, &vec!["safe"], MAX_JSON_BYTES).unwrap();
        assert_eq!(fs::read(&backups[0]).unwrap(), b"{truncated");
        assert_eq!(read_or_default::<Vec<String>>(&path, MAX_JSON_BYTES).unwrap().value, vec!["safe"]);
    }

    #[test]
    fn oversized_state_is_quarantined_and_future_writes_are_bounded() {
        let dir = TestDir::new();
        let path = dir.file();
        fs::write(&path, vec![b'x'; 128]).unwrap();
        let values: Loaded<Vec<String>> = read_or_default(&path, 32).unwrap();
        assert!(values.value.is_empty());
        assert!(values.quarantined);
        assert!(!path.exists());
        assert!(write_atomic(&path, &vec!["too-long-item"], 5).is_err());
        assert!(!path.exists());
    }

    #[test]
    fn absent_file_is_empty_but_directory_is_an_error() {
        let dir = TestDir::new();
        let path = dir.file();
        let loaded = read_or_default::<Vec<String>>(&path, MAX_JSON_BYTES).unwrap();
        assert!(loaded.value.is_empty());
        assert!(!loaded.quarantined);
        assert!(read_or_default::<Vec<String>>(&dir.0, MAX_JSON_BYTES).is_err());
        assert!(write_atomic(&dir.0, &vec!["x"], MAX_JSON_BYTES).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_state_does_not_follow_or_overwrite_other_files() {
        let dir = TestDir::new();
        let secret = dir.0.join("outside.json");
        fs::write(&secret, b"[\"protected\"]").unwrap();
        let path = dir.file();
        std::os::unix::fs::symlink(&secret, &path).unwrap();
        assert!(read_or_default::<Vec<String>>(&path, MAX_JSON_BYTES).is_err());
        assert!(write_atomic(&path, &vec!["tampered"], MAX_JSON_BYTES).is_err());
        assert_eq!(fs::read(&secret).unwrap(), b"[\"protected\"]");
    }
    #[test]
    fn given_exact_json_length_when_writing_then_limit_comparison_is_inclusive() {
        let dir = TestDir::new();
        let path = dir.file();
        let payload = Vec::<String>::new();
        let length = serde_json::to_vec_pretty(&payload).unwrap().len() as u64;
        assert_eq!(length,2);
        assert!(write_atomic(&path,&payload,length).is_ok());
        assert_eq!(read_or_default::<Vec<String>>(&path,length).unwrap().value,payload);
        assert!(write_atomic(&path,&payload,length - 1).is_err());
        assert_eq!(fs::read(&path).unwrap(),b"[]");
    }

    #[test]
    fn given_oversized_replacement_when_writing_then_previous_good_state_is_not_lost() {
        let dir=TestDir::new();
        let path=dir.file();
        write_atomic(&path,&vec!["old"],MAX_JSON_BYTES).unwrap();
        let before=fs::read(&path).unwrap();
        let too_big = vec!["x".repeat(200)];
        assert!(write_atomic(&path,&too_big,20).is_err());
        assert_eq!(fs::read(&path).unwrap(),before);
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(),1);
    }

    #[test]
    fn given_missing_parent_directory_when_writing_then_failure_does_not_leave_temp_or_fake_success() {
        let dir=TestDir::new();
        let missing=dir.0.join("absent");
        let path=missing.join("events.json");
        assert!(write_atomic(&path,&vec!["x"],MAX_JSON_BYTES).is_err());
        assert!(!path.exists());
        assert!(!missing.exists());
    }

    #[test]
    fn given_truncated_json_when_recovering_then_a_real_backup_precedes_new_state() {
        let dir=TestDir::new();
        let path=dir.file();
        fs::write(&path,b"{\"entries\":[").unwrap();
        let result: Loaded<Vec<String>>=read_or_default(&path,MAX_JSON_BYTES).unwrap();
        assert!(result.quarantined);
        assert!(result.value.is_empty());
        assert!(!path.exists());
        let backups:Vec<_>=fs::read_dir(&dir.0).unwrap()
          .map(|item|item.unwrap().path())
          .filter(|p|p.file_name().unwrap().to_string_lossy().contains(".corrupt."))
          .collect();
        assert_eq!(backups.len(),1);
        assert_eq!(fs::read(&backups[0]).unwrap(),b"{\"entries\":[");
        write_atomic(&path,&vec!["fresh"],MAX_JSON_BYTES).unwrap();
        assert_eq!(fs::read(&backups[0]).unwrap(),b"{\"entries\":[");
    }

    #[test]
    fn given_multiple_independent_invalid_files_when_quarantining_then_each_backup_is_preserved() {
        let dir=TestDir::new();
        let path=dir.file();
        for attempt in 0..3 {
            let corrupt=format!("not-json-{attempt}");
            fs::write(&path,corrupt.as_bytes()).unwrap();
            let result:Loaded<Vec<String>>=read_or_default(&path,MAX_JSON_BYTES).unwrap();
            assert!(result.quarantined);
            assert!(!path.exists());
        }
        let backups:Vec<_>=fs::read_dir(&dir.0).unwrap()
          .map(|item|item.unwrap().path()).collect();
        assert_eq!(backups.len(),3);
        let mut contents=backups.iter()
          .map(|path|fs::read_to_string(path).unwrap()).collect::<Vec<_>>();
        contents.sort();
        assert_eq!(contents,vec!["not-json-0","not-json-1","not-json-2"]);
    }

}
