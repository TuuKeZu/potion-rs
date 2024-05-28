use std::{collections::VecDeque, fs, io, path::Path};

use crate::file_system::resolve_relative_path;

pub fn construct_routing_system(path: &str) -> io::Result<String> {
    let mut ts = String::from("mod routing { ");
    let mut l: Vec<VecDeque<String>> = vec![];
    construct_import_tree(path, &mut ts, &mut l)?;
    ts += "}";

    ts += "\n";
    ts += &construct_router_tree(&mut l)?;

    Ok(ts)
}

pub fn construct_import_tree(
    dir: &str,
    s: &mut String,
    l: &mut Vec<VecDeque<String>>,
) -> io::Result<()> {
    if Path::new(dir).is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                *s += &format!("pub mod {} {{", entry.file_name().to_str().unwrap());
                construct_import_tree(&path.to_str().unwrap().to_string(), s, l)?;
                *s += "} ";
            } else {
                if path.extension().unwrap() == "rs" {
                    let file = entry.file_name().to_str().unwrap().replace(".rs", "");
                    *s += &format!(" pub mod {}; ", &file);
                    dbg!(&path);
                    l.push(resolve_relative_path(path, true)?);
                }
            }
        }
    }
    Ok(())
}

pub fn construct_router_tree(l: &mut Vec<VecDeque<String>>) -> io::Result<String> {
    let mut ts = String::from("fn router(hb: Arc<Handlebars<'static>>, pool: Arc<Pool<Postgres>>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone { ");

    let mut tree_list: Vec<&VecDeque<String>> = l.iter().filter(|tree| tree.iter().last().unwrap() == "index").collect();
    
    if let Some(tree) = tree_list.pop() {
        if tree.len() <= 0 {
            panic!("Failed to construct router tree. Tried to link file with invalid path")
        }
        if tree.iter().last().unwrap() == "index" {
            for (i, route) in tree.iter().enumerate() {
                dbg!(&route, &i);
                if i == 0 {
                    ts += &format!("warp::path(\"{}\").and(", route)
                } else if i == tree.len() - 1 {
                    ts += &format!("routing::{}::initialize( potion::Context::new(hb.clone(), pool.clone(), &[{}]) )", resolve_import_path(tree.clone()), tree.iter().map(|r| format!("\"{r}\"")).collect::<Vec<String>>().join(", "));
                } else {
                    ts += &format!("warp::path(\"{}\").and(", route)
                }
            }

            ts += &")".repeat(tree.len() - 1);
        }
    } else {
        return Ok(ts);
    }

    for tree in tree_list {
        dbg!(&tree);
        if tree.iter().last().unwrap() == "index" {
            for (i, route) in tree.iter().enumerate() {
                if i == 0 {
                    ts += &format!(".or(warp::path(\"{}\").and(", route)
                } else if i == tree.len() - 1 {
                    ts += &format!("routing::{}::initialize( potion::Context::new(hb.clone(), pool.clone(), &[{}]) )", resolve_import_path(tree.clone()), tree.iter().map(|r| format!("\"{r}\"")).collect::<Vec<String>>().join(", "));
                } else {
                    ts += &format!("warp::path(\"{}\").and(", route)
                }
            }

            ts += &")".repeat(tree.len());
        }
    }

    ts += " }";

    Ok(ts)
}

fn resolve_import_path(tree: VecDeque<String>) -> String {
    Vec::from(tree).join("::")
}
