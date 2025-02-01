use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

pub struct Router {
    routes: HashMap<String, fn() -> String>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn route(mut self, path: &str, page: fn() -> String) -> Self {
        if path.is_empty() {
            panic!("Paths must start with a `/`. Use \"/\" for root routes");
        } else if !path.starts_with('/') {
            panic!("Paths must start with a `/`");
        }

        if let Some(_) = self.routes.insert(path.to_string(), page) {
            panic!("Overlapping method route. Handler for `{path}` already exists");
        }

        self
    }

    pub fn fallback(self, page: fn() -> String) -> Self {
        let path = "/404";
        if self.routes.contains_key(path) {
            panic!("Overlapping method route. Fallback handler already exists");
        }

        self.route(path, page)
    }

    pub fn render(&self, export_path: &Path) -> io::Result<()> {
        fs::create_dir_all(export_path)?;

        for (path, page) in &self.routes {
            let page_path = match path.strip_prefix("/").unwrap() {
                "" => "index",
                path => path,
            };

            let mut export_path = export_path.to_path_buf();
            export_path.push(page_path);
            export_path.set_extension("html");

            fs::create_dir_all(export_path.parent().unwrap())?;
            fs::write(export_path, page())?;
        }

        Ok(())
    }
}
