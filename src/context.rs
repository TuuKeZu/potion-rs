use std::sync::Arc;

use handlebars::Handlebars;
use serde::Serialize;
use serde_json::{json, Value};

use crate::utility::merge;



#[derive(Debug, Clone)]
pub struct Context {
    pub hb: Arc<Handlebars<'static>>,
    local_directory: DirLink,
    global_directory: DirLink,
}

impl Context {

    pub fn new(hb: Arc<Handlebars<'static>>, path: &[&str]) -> Self {
        Self {
            hb,
            local_directory: DirLink::from(path),
            global_directory: DirLink::root(),
        }
    }

    pub fn template(&self) -> String {
        self.local_directory.get_template("page")
    }

    pub fn local_dir(&self) -> &DirLink {
        &self.local_directory
    }

    pub fn global_dir(&self) -> &DirLink {
        &self.global_directory
    }

    pub fn render<T>(&self, page: Page<T>) -> impl warp::Reply where T: Serialize {
        self.render_template(&self.template(), page)
    }

    pub fn render_template<T>(&self, template_name: &str, page: Page<T>) -> impl warp::Reply where T: Serialize {
        let mut value = json!(page.value);
        merge(&mut value, page.generate_import_tree(&self));

        let render = self.hb
            .render(template_name, &value)
            .unwrap_or_else(|err| err.to_string());

        warp::reply::html(render)
    }

    pub fn import_styling(&self, name: &str) -> String {
        self.local_directory.get_static_file(name)
    }

}

#[derive(Debug, Clone)]
pub struct Page<T: Serialize> {
    pub value: T,
    local_style_tree: Vec<String>,
    global_style_tree: Vec<String>,
}

impl<T> Page<T> where T: Serialize {
    pub fn new(value: T) -> Self {
        Self {
            value,
            local_style_tree: vec![],
            global_style_tree: vec![],
        }
    }

    pub fn default(value: T) -> Self {
        Self {
            value,
            local_style_tree: vec![String::from("page.css")],
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

    pub fn generate_import_tree(&self, context: &Context) -> Value {
        let mut local_tree = self.local_style_tree.iter().filter_map(|file| {
           serde_json::to_value(context.local_directory.get_static_file(file)).ok()
        }).collect::<Vec<Value>>();

        let mut global_tree = self.global_style_tree.iter().filter_map(|file| {
            serde_json::to_value(context.global_directory.get_static_file(file)).ok()
        }).collect::<Vec<Value>>();
        
        local_tree.append(&mut global_tree);
        let tree = local_tree;

        json!({
            "STYLE_IMPORTS": tree,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DirLink {
    path: Option<String>,
}

impl DirLink {

    pub fn root() -> Self {
        Self {
            path: None
        }
    }

    pub fn get_static_file(&self, file_path: &str) -> String {
        match &self.path {
            Some(path) => format!("/static/{}::{}", path, file_path),
            None => format!("/static/{}", file_path),
        }
    }

    pub fn get_template(&self, template_name: &str) -> String {
        match &self.path {
            Some(path) => format!("{}::{}", path, template_name),
            None => format!("{}", template_name),
        }
    }

    pub fn get_relative_path(&self) -> &Option<String> {
        &self.path
    }
}

impl From<&[&str]> for DirLink {
    fn from(value: &[&str]) -> Self {
        let value = Vec::from_iter(value.iter().filter(|v| **v != "page").map(|p| p.to_string()));
        Self {
            path: Some(value.join("::")),
        }
    }
}