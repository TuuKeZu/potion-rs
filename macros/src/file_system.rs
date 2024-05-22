use std::{
    collections::VecDeque,
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

pub fn resolve_relative_path(path: PathBuf, truncate: bool) -> io::Result<VecDeque<String>> {
    let mut l: VecDeque<String> = VecDeque::new();
    let mut path = path;

    while !path.ends_with("routing") {
        if let Some(a) = path.file_name().map(|p| p.to_str().unwrap()) {
            let file = if truncate {
                if let Some(e) = path.extension() {
                    let extension = format!(".{}", e.to_str().unwrap());
                    a.to_string().replace(&extension, "")
                } else {
                    a.to_string()
                }
            } else {
                a.to_string()
            };

            l.push_front(file);
        }

        path.pop();
    }

    Ok(l)
}
