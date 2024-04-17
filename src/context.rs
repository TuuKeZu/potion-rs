use std::sync::Arc;

use handlebars::Handlebars;
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::Postgres;
use warp::Filter;

use sqlx::Pool;

use crate::utility::merge;



#[derive(Debug, Clone)]
pub struct Context {
    pub hb: Arc<Handlebars<'static>>,
    pub db: Arc<Pool<Postgres>>,
    pub storage: DirLink,
}

impl Context {

    pub fn with_db(&self) -> impl Filter<Extract = (Arc<Pool<Postgres>>,), Error = std::convert::Infallible> + Clone {
        let db = self.db.clone();
        warp::any().map(move || db.clone())
    }

    pub fn with_hb(&self) -> impl Filter<Extract = (Arc<Handlebars<'static>>,), Error = std::convert::Infallible> + Clone {
        let hb = self.hb.clone();
        warp::any().map(move || hb.clone())
    }

    pub fn with_storage(self) -> impl Filter<Extract = (DirLink,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || self.storage.clone())
    }

    pub fn with_context(self) -> impl Filter<Extract = ((Arc<Pool<Postgres>>, Arc<Handlebars<'static>>, DirLink), ), Error = std::convert::Infallible> + Clone {
        let db = self.db.clone();
        let hb = self.hb.clone();
        let storage = self.storage.clone();

        warp::any().map(move || (db.clone(), hb.clone(), storage.clone()) )
    }


    pub fn new(hb: Arc<Handlebars<'static>>, db: Arc<Pool<Postgres>>, path: &[&str]) -> Self {
        Self {
            hb,
            db,
            storage: DirLink::from(path),
        }
    }

    pub fn template(&self) -> String {
        self.storage.get_template("index")
    }


    pub fn import_styling(&self, name: &str) -> String {
        self.storage.get_local_file(name)
    }

}

#[derive(Debug, Clone)]
pub struct Page<T: Serialize> {
    pub value: T,
    template_tree: String,
    local_style_tree: Vec<String>,
    global_style_tree: Vec<String>,
}

impl<T> Page<T> where T: Serialize {
    pub fn new(template: &str, value: T) -> Self {
        Self {
            value,
            template_tree: template.to_string(),
            local_style_tree: vec![],
            global_style_tree: vec![],
        }
    }

    pub fn default(template: &str, value: T) -> Self {
        Self {
            value,
            template_tree: String::from(template),
            local_style_tree: vec![String::from("index.css")],
            global_style_tree: vec![String::from("index.css")],
        }
    }

    pub fn with_local_styles(mut self, styles: &[&str]) -> Self {
        self.local_style_tree = Vec::from_iter(styles.iter().map(|s| s.to_string()));
        self
    }

    pub fn with_global_styles(mut self, styles: &[&str]) -> Self {
        self.global_style_tree = Vec::from_iter(styles.iter().map(|s| s.to_string()));
        self
    }

    pub fn generate_import_tree(&self, storage: &DirLink) -> Value {
        let mut local_tree = self.local_style_tree.iter().filter_map(|file| {
           serde_json::to_value(storage.get_local_file(file)).ok()
        }).collect::<Vec<Value>>();

        let mut global_tree = self.global_style_tree.iter().filter_map(|file| {
            serde_json::to_value(storage.get_static_file(file)).ok()
        }).collect::<Vec<Value>>();
        
        local_tree.append(&mut global_tree);
        let tree = local_tree;

        json!({
            "STYLE_IMPORTS": tree,
        })
    }

    pub fn render(self, hb: Arc<Handlebars<'static>>, storage: &DirLink) -> impl warp::Reply {
        let tree = &self.template_tree.clone();
        self.render_template(hb, storage, &tree)
    }

    pub fn render_template(self, hb: Arc<Handlebars<'static>>, storage: &DirLink, template_name: &str) -> impl warp::Reply {
        let mut value = json!(self.value);
        let tree = self.generate_import_tree(storage);
        merge(&mut value, tree);

        let render = hb
            .render(template_name, &self.value)
            .unwrap_or_else(|err| err.to_string());

        warp::reply::html(render)
    }

}



#[derive(Debug, Clone)]
pub struct DirLink {
    path: String,
}

impl DirLink {

    pub fn get_static_file(&self, file_path: &str) -> String {
        format!("/static/{}::{}", self.path, file_path)
    }

    pub fn get_local_file(&self, file_path: &str) -> String {
        format!("/static/{}", file_path)
    }

    pub fn template(&self) -> String {
        self.get_template("index")
    }

    pub fn get_template(&self, template_name: &str) -> String {
        format!("{}::{}", self.path, template_name)
    }

    pub fn get_static_template(&self, template_name: &str) -> String {
        format!("{}", template_name)
    }

    pub fn get_relative_path(&self) -> &String {
        &self.path
    }
}

impl From<&[&str]> for DirLink {
    fn from(value: &[&str]) -> Self {
        let value = Vec::from_iter(value.iter().filter(|v| **v != "index").map(|p| p.to_string()));
        Self {
            path: value.join("::"),
        }
    }
}