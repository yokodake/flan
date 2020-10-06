use crate::debug;
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
    let mut its = path.as_ref().components();
    // special case for root of path
    for it in &mut its {
        match it {
            Component::CurDir => continue,
            Component::ParentDir => {
                depth -= 1;
            }
            _ => {}
        }
        normal.push(it);
        break;
    }
    // rest of path
    for it in its {
        debug!("{:?}", normal);
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
