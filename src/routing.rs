use std::{
    fs,
    fs::DirEntry,
    io,
    path::{Path, PathBuf},
};

use handlebars::Handlebars;
use warp::{filters::BoxedFilter, Filter};

use crate::file_system::{resolve_relative_path, visit_dirs};

pub fn initialize_routing(
    path: &str,
    dev: bool,
) -> io::Result<(Handlebars<'static>, BoxedFilter<(warp::filters::fs::File,)>)> {
    let mut file_map = map_routing_tree(path)?;

    #[cfg(feature = "typescript")]
    typescript_code_gen(&PathBuf::from(path), &mut file_map)?;

    let static_router = link_static_files(&file_map)
        .or(link_static_dir(path.into()))
        .unify()
        .boxed();

    Ok((load_templates(&file_map, dev)?, static_router))
}

fn map_routing_tree(path: &str) -> io::Result<Vec<(String, PathBuf)>> {
    let mut l: Vec<DirEntry> = vec![];
    visit_dirs(
        &Path::new(path).join(Path::new("routing")),
        &mut l,
        &["hbs", "css", "ts"],
    )?;
    visit_dirs(
        &Path::new(path).join(Path::new("static")),
        &mut l,
        &["hbs", "css", "ts"],
    )?;

    let file_map: Vec<(String, PathBuf)> = l
        .iter()
        .filter_map(|file| {
            if let Ok(f) = resolve_relative_path(file.path(), false) {
                Some((f, file.path()))
            } else {
                None
            }
        })
        .map(|(route, path)| (Vec::from(route).join("."), path))
        .collect();

    Ok(file_map)
}

pub fn load_templates(
    file_map: &Vec<(String, PathBuf)>,
    dev: bool,
) -> io::Result<Handlebars<'static>> {
    let mut hb = Handlebars::new();

    hb.set_dev_mode(dev);

    for (template, path) in file_map
        .iter()
        .filter(|(t, _)| t.ends_with(".hbs"))
        .map(|(t, p)| (t.replace(".hbs", "").replace(".", "::"), p))
    {
        hb.register_template_file(&template, path).unwrap();
    }

    Ok(hb)
}

pub fn link_static_files(l: &Vec<(String, PathBuf)>) -> BoxedFilter<(warp::filters::fs::File,)> {
    let root = warp::path("static")
        .and(warp::path::end())
        .and(warp::fs::file(""))
        .boxed();

    let router = l
        .iter()
        .filter(|(t, _)| t.ends_with(".css") || t.ends_with(".js") || t.ends_with(".js:map"))
        .map(|(t, p)| (t, p.clone()))
        .fold(root, |f, (r, p)| {
            let name = resolve_static_file_name(r);

            let g = warp::path("static")
                .and(warp::path(name).and(warp::fs::file(p)))
                .boxed();

            f.or(g).unify().boxed()
        });

    router
}

pub fn resolve_static_file_name(tree: &String) -> String {
    let tree = tree
        .split(".")
        .map(|t| t.to_string())
        .collect::<Vec<String>>();
    let (route, extension) = tree.split_at(tree.len() - 1);
    format!("{}.{}", route.join("::"), extension.first().unwrap()).replace("js:map", "js.map")
}

pub fn link_static_dir(path: PathBuf) -> BoxedFilter<(warp::fs::File,)> {
    warp::path("static")
        .and(warp::fs::dir(path.join(Path::new("static"))))
        .boxed()
}

#[cfg(feature = "typescript")]
pub fn typescript_code_gen(
    routing_path: &Path,
    file_map: &mut Vec<(String, PathBuf)>,
) -> io::Result<()> {
    use crate::wsc::ts_to_js;
    use minify_js::{minify, Session, TopLevelMode};

    let session = Session::new();
    let mut script_map = vec![];

    for (route, path) in file_map
        .iter()
        .filter(|(t, _)| t.ends_with(".ts"))
        .map(|(t, p)| (format!("{}.js", t.replace(".ts", "")), p))
    {
        let filename = path
            .file_name()
            .map(|s| s.to_str().unwrap_or("unknown"))
            .unwrap_or("unknown");
        let map_route = route.replace(".js", ".js:map");
        let content = fs::read_to_string(path)?;
        let output_path = routing_path.join("../dist").join(route.clone());
        let map_output_path = routing_path
            .join("../dist")
            .join(map_route.replace(".js:map", ".js.map").clone());

        let (out, map) = ts_to_js(filename, &content).expect("Failed to compile ts file");
        let out = out.as_bytes();
        let map = map.as_bytes();

        let mut out_buffer = Vec::new();

        minify(&session, TopLevelMode::Global, out, &mut out_buffer)
            .expect("Failed to minify generated js");

        let source_map_ref = format!(
            "\n//# sourceMappingURL=/static/{}",
            resolve_static_file_name(&map_route)
        );

        fs::write(
            output_path.clone(),
            &[out_buffer, source_map_ref.as_bytes().to_vec()].concat(),
        )?;
        fs::write(map_output_path.clone(), map)?;

        script_map.push((route, output_path));
        script_map.push((map_route, map_output_path));
        dbg!(&script_map);
    }

    file_map.append(&mut script_map);

    Ok(())
}
