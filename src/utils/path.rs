use std::env::current_dir;
use std::io::Result;
use std::path::{Path, PathBuf};

/// normalizes the path without following symlinks.
/// this means that some `..` are not normalized away:
/// * parents of relative root (if root is relative)
/// * parents of symlinks
pub fn normalize_path<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    use std::path::Component;
    let mut normal = PathBuf::new();
    let mut depth = if path.as_ref().is_relative() {
        0
    } else {
        i32::MAX / 2
    };
    for it in path.as_ref().components() {
        depth = if normal.symlink_metadata()?.file_type().is_symlink() {
            0
        } else {
            depth
        };
        match it {
            Component::CurDir => continue,
            Component::ParentDir => {
                if depth < 1 {
                    normal.push(it)
                } else {
                    depth -= 1;
                    normal.pop();
                }
            }
            _ => {
                depth += 1;
                normal.push(it);
            }
        }
    }
    Ok(normal)
}
