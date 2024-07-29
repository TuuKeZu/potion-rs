use std::sync::Arc;

use handlebars::Handlebars;
use minify_html::{minify, Cfg};
use serde::Serialize;
use serde_json::{json, Value};


use crate::{storage::Storage, utility::merge};

#[derive(Clone, PartialEq, PartialOrd)]
pub enum PageValueScope {
    ServerSide,
    ClientSide
}

#[derive(Clone)]
struct PageValue<T: Serialize> {
    value: T,
    scope: PageValueScope
}

impl<T :Serialize> PageValue<T> {
    fn new(value: T, scope: PageValueScope) -> Self {
        Self {
            value,
            scope
        }
    }
}


#[derive(Clone)]
pub struct Page<T: Serialize> {
    values: Vec<PageValue<T>>,

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
    pub fn new(template: &str, value: T, scope: PageValueScope) -> Self {
        Self {
            values: vec![PageValue::new(value, scope)],
            template: Some(template.to_string()),
            parent: None,
            child: None,
            navigation: None,
            local_style_tree: vec![],
            global_style_tree: vec![],
            local_script_tree: vec![],
            global_script_tree: vec![],
            _cfg: Cfg::default()
        }
    }

    pub fn default(storage: &Storage) -> Self {
        Self {
            values: vec![],
            template: Some(storage.get_template()),
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

    pub fn with_value(mut self, value: T, scope: PageValueScope) -> Self {
        self.values.push(PageValue::new(value, scope));
        self
    }

    pub fn with_navigation(mut self, storage: &Storage) -> Self {
        self.navigation = Some(storage.construct_navigation());
        self
    }

    pub fn generate_import_tree(&self, storage: &Storage) -> Value {
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

        if self.values.iter().find(|value| value.scope == PageValueScope::ClientSide).is_some() {
            let client_value_tree = self.values.iter().filter_map(|value| match value.scope {
                PageValueScope::ServerSide => None,
                PageValueScope::ClientSide => Some(json!(value.value)),
            }).fold(json!("{}"), |mut a, v| {
                merge(&mut a, v);
                a
            });

            export["CLIENT_SIDE_VALUES"] = Value::String(format!("const potion = {};", client_value_tree.to_string()));
        }

        export
    }

    pub fn render(self, hb: &Arc<Handlebars<'static>>, storage: &Storage) -> impl warp::Reply {
        let template_name = match &self.template {
            Some(name) => name.clone(),
            None => panic!("Cannot render a page without a template"),
        };

        self.render_template(hb, storage, &template_name)
    }

    pub fn render_template(
        self,
        hb: &Arc<Handlebars<'static>>,
        storage: &Storage,
        template_name: &str,
    ) -> impl warp::Reply {
        let tree = self.generate_import_tree(storage);

        let server_value_tree = self.values.iter().filter_map(|value| match value.scope {
            PageValueScope::ServerSide => Some(json!(value.value)),
            PageValueScope::ClientSide => None,
        }).fold(tree, |mut a, v| {
            merge(&mut a, v);
            a
        });
    
        let template_name = match &self.child {
            Some(child) => child.clone(),
            None => template_name.to_string(),
        };


        let render = hb
            .render(&template_name, &server_value_tree)
            .unwrap_or_else(|err| err.to_string());

        let minified = minify(render.as_bytes(), &self._cfg);
        warp::reply::html(minified)
    }
}