#[derive(Debug, Clone)]
pub struct Storage {
    path: String,
}

impl Storage {
    pub fn get_static_file(&self, path: &str) -> String {
        format!("/static/static::{}", path)
    }

    pub fn get_local_file(&self, name: &str) -> String {
        format!("/static/routing::{}::{}", self.path, name)
    }

    pub fn get_template(&self) -> String {
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

impl From<&[&str]> for Storage {
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
