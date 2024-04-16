use std::{fs::DirEntry, io, path::{Path, PathBuf}, sync::Arc};

use handlebars::Handlebars;
use warp::{filters::BoxedFilter, Filter};

use crate::file_system::{resolve_relative_path, visit_dirs};



pub fn initialize_routing(path: &str) -> io::Result<(Arc<Handlebars<'static>>, BoxedFilter<(warp::filters::fs::File,)>)> {
    let file_map = map_routing_tree(path)?;

    let router = link_static_files(&file_map).or(link_static_dir(path.into())).unify().boxed();

    Ok((load_templates(&file_map)?, router))
}


fn map_routing_tree(path: &str) -> io::Result<Vec<(String, PathBuf)>> {
    let mut l: Vec<DirEntry> = vec![];
    visit_dirs(Path::new(path), &mut l, &["html", "css"])?;
    let file_map: Vec<(String, PathBuf)> = l.iter().filter_map(|file| {
        if let Ok(f) = resolve_relative_path(file.path(), false) {
            Some((f, file.path()))
        } else {
            None
        }
    }).map(|(route, path)| (Vec::from(route).join("."), path)).collect();
    
    Ok(file_map)
}

pub fn load_templates(file_map: &Vec<(String, PathBuf)>) -> io::Result<Arc<Handlebars<'static>>> {
    let mut hb = Handlebars::new();

    for (template, path) in file_map.iter().filter(|(t, _)| t.ends_with(".html")).map(|(t, p)| (t.replace(".html", "").replace(".", "::"), p)) {
        hb.register_template_file(&template, path).unwrap();
    }

    Ok(Arc::new(hb))
}

pub fn link_static_files(l: &Vec<(String, PathBuf)>) -> BoxedFilter<(warp::filters::fs::File,)> {
    let root = warp::path("static").and(warp::path::end()).and(warp::fs::file("")).boxed();

    let router = l.iter().filter(|(t, _)| t.ends_with(".css")).map(|(t, p)| (t.split(".").map(|t| t.to_string()).collect::<Vec<String>>(), p.clone())).fold(root, |f, (r, p)| {
        let (route, extension) = r.split_at(r.len() - 1);
        let name = format!("{}.{}", route.join("::"), extension.first().unwrap());
        let g = warp::path("static").and(warp::path(name).and(warp::fs::file(p))).boxed();
        
        f.or(g).unify().boxed()
    });

    router
}

pub fn link_static_dir(path: PathBuf) -> BoxedFilter<(warp::fs::File,)> {
    warp::path("static").and(warp::fs::dir(path)).boxed()
}