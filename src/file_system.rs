use std::{
    collections::VecDeque,
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

pub fn visit_dirs(dir: &Path, l: &mut Vec<DirEntry>, extensions: &[&str]) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, l, extensions)?;
            } else {
                if extensions.contains(&entry.path().extension().unwrap().to_str().unwrap()) {
                    l.push(entry);
                }
            }
        }
    }
    Ok(())
}

pub fn resolve_relative_path(path: PathBuf, truncate: bool) -> io::Result<VecDeque<String>> {
    let mut l: VecDeque<String> = VecDeque::new();
    let mut path = path;

    while !(path.ends_with("routing") || path.ends_with("static")) {
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

    if let Some(a) = path.file_name().map(|p| p.to_str().unwrap()) {
        l.push_front(a.to_string());
    }

    Ok(l)
}
