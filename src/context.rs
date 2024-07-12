use std::sync::Arc;

use handlebars::Handlebars;
use minify_html::{minify, Cfg};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::Postgres;
use warp::Filter;

use sqlx::Pool;

use crate::utility::merge;

pub type ContextRef = (Pool<Postgres>, Arc<Handlebars<'static>>, DirLink);

#[derive(Debug, Clone)]
pub struct Context {
    pub hb: Arc<Handlebars<'static>>,
    pub db: Pool<Postgres>,
    pub storage: DirLink,
}

impl Context {
    pub fn with_db(
        &self,
    ) -> impl Filter<Extract = (Pool<Postgres>,), Error = std::convert::Infallible> + Clone
    {
        let db = self.db.clone();
        warp::any().map(move || db.clone())
    }

    pub fn with_hb(
        &self,
    ) -> impl Filter<Extract = (Arc<Handlebars<'static>>,), Error = std::convert::Infallible> + Clone
    {
        let hb = self.hb.clone();
        warp::any().map(move || hb.clone())
    }

    pub fn with_storage(
        self,
    ) -> impl Filter<Extract = (DirLink,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || self.storage.clone())
    }

    pub fn with_context(
        &self,
    ) -> impl Filter<Extract = (ContextRef,), Error = std::convert::Infallible> + Clone {
        let db = self.db.clone();
        let hb = self.hb.clone();
        let storage = self.storage.clone();

        warp::any().map(move || (db.clone(), hb.clone(), storage.clone()))
    }

    pub fn new(hb: Arc<Handlebars<'static>>, db: Pool<Postgres>, path: &[&str]) -> Self {
        Self {
            hb,
            db,
            storage: DirLink::from(path),
        }
    }

    pub fn import_styling(&self, name: &str) -> String {
        self.storage.get_local_file(name)
    }
}

#[derive(Clone)]
pub struct Page<T: Serialize> {
    value: Option<T>,
    template: Option<String>,
    parent: Option<String>,
    child: Option<String>,
    navigation: Option<Vec<(String, String)>>,
    local_style_tree: Vec<String>,
    global_style_tree: Vec<String>,
    local_script_tree: Vec<String>,
    global_script_tree: Vec<String>,
    _cfg: Cfg
}

impl<T> Page<T>
where
    T: Serialize,
{
    pub fn new(template: &str, value: T) -> Self {
        Self {
            value: Some(value),
            template: Some(template.to_string()),
            parent: None,
            child: None,
            navigation: None,
            local_style_tree: vec![],
            global_style_tree: vec![],
            local_script_tree: vec![],
            global_script_tree: vec![],
            _cfg: Cfg::new()
        }
    }

    pub fn default(storage: &DirLink) -> Self {
        Self {
            value: None,
            template: Some(storage.template()),
            parent: Some(storage.get_static_template("base")),
            child: None,
            navigation: Some(storage.construct_navigation()),
            local_style_tree: vec![String::from("index.css")],
            global_style_tree: vec![String::from("index.css")],
            local_script_tree: vec![String::from("index.js")],
            global_script_tree: vec![String::from("index.js")],
            _cfg: Cfg::new()
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

    pub fn with_local_scripts(mut self, styles: &[&str]) -> Self {
        self.local_script_tree = Vec::from_iter(styles.iter().map(|s| s.to_string().replace(".ts", ".js")));
        self
    }

    pub fn with_global_scripts(mut self, styles: &[&str]) -> Self {
        self.global_script_tree = Vec::from_iter(styles.iter().map(|s| s.to_string().replace(".ts", ".js")));
        self
    }

    pub fn with_parent(mut self, parent_path: &str) -> Self {
        self.parent = Some(parent_path.to_string());
        self
    }

    pub fn with_child(mut self, child: &str) -> Self {
        self.child = Some(child.to_string());
        self
    }

    pub fn with_template(mut self, template_path: &str) -> Self {
        self.template = Some(template_path.to_string());
        self
    }

    pub fn with_value(mut self, value: T) -> Self {
        self.value = Some(value);
        self
    }

    pub fn with_navigation(mut self, storage: &DirLink) -> Self {
        self.navigation = Some(storage.construct_navigation());
        self
    }

    pub fn generate_import_tree(&self, storage: &DirLink) -> Value {
        let mut local_styles = self
            .local_style_tree
            .iter()
            .filter_map(|file| serde_json::to_value(storage.get_local_file(file)).ok())
            .collect::<Vec<Value>>();

        let mut local_scripts = self
            .local_script_tree
            .iter()
            .filter_map(|file| serde_json::to_value(storage.get_local_file(file)).ok())
            .collect::<Vec<Value>>();

        let mut global_styles = self
            .global_style_tree
            .iter()
            .filter_map(|file| serde_json::to_value(storage.get_static_file(file)).ok())
            .collect::<Vec<Value>>();

        let mut global_scripts = self
            .global_script_tree
            .iter()
            .filter_map(|file| serde_json::to_value(storage.get_static_file(file)).ok())
            .collect::<Vec<Value>>();

        local_styles.append(&mut global_styles);
        local_scripts.append(&mut global_scripts);

        let style_tree = local_styles;
        let scripts_tree = local_scripts;

        let mut export = json!({
            "STYLE_IMPORTS": style_tree,
            "SCRIPT_IMPORTS": scripts_tree
        });

        match &self.navigation {
            Some(navigation) => {
                export["NAVIGATION"] = json!(*navigation);
            }
            None => {}
        }

        match &self.parent {
            Some(parent) => {
                export["PARENT"] = json!(*parent);
            }
            None => {}
        }

        match &self.child {
            Some(_) => {
                export["INNER_PARENT"] = json!(self.template.clone().unwrap());
            }
            None => {}
        }

        export
    }

    pub fn render(self, hb: Arc<Handlebars<'static>>, storage: &DirLink) -> impl warp::Reply {
        let template_name = match &self.template {
            Some(name) => name.clone(),
            None => panic!("Cannot render a page without a template"),
        };

        self.render_template(hb, storage, &template_name)
    }

    pub fn render_template(
        self,
        hb: Arc<Handlebars<'static>>,
        storage: &DirLink,
        template_name: &str,
    ) -> impl warp::Reply {
        let mut value = json!(self.value);
        let tree = self.generate_import_tree(storage);

        let template_name = match &self.child {
            Some(child) => child.clone(),
            None => template_name.to_string(),
        };

        merge(&mut value, tree);

        let render = hb
            .render(&template_name, &value)
            .unwrap_or_else(|err| err.to_string());

        let minified = minify(render.as_bytes(), &self._cfg);
        warp::reply::html(minified)
    }
}

#[derive(Debug, Clone)]
pub struct DirLink {
    path: String,
}

impl DirLink {
    pub fn get_static_file(&self, path: &str) -> String {
        format!("/static/static::{}", path)
    }

    pub fn get_local_file(&self, name: &str) -> String {
        format!("/static/routing::{}::{}", self.path, name)
    }

    pub fn template(&self) -> String {
        self.get_local_template("index")
    }

    pub fn get_local_template(&self, name: &str) -> String {
        format!("routing::{}::{}", self.path, name)
    }

    pub fn get_static_template(&self, path: &str) -> String {
        format!("static::{}", path)
    }

    pub fn get_relative_path(&self) -> &String {
        &self.path
    }

    pub fn construct_navigation(&self) -> Vec<(String, String)> {
        let a: Vec<String> = self.path.split("::").map(|n| n.to_string()).collect();
        let mut b: Vec<(String, String)> = vec![];

        for i in 0..a.len() {
            let c = a
                .iter()
                .take(i + 1)
                .fold(String::new(), |a, v| a + format!("/{v}").as_str());

            let last = &c.split("/").last().map(|s| s.to_string()).unwrap();
            b.push((c, last.to_owned()));
        }

        b
    }
}

impl From<&[&str]> for DirLink {
    fn from(value: &[&str]) -> Self {
        let value = Vec::from_iter(
            value
                .iter()
                .filter(|v| **v != "index")
                .map(|p| p.to_string()),
        );
        Self {
            path: value.join("::"),
        }
    }
}
