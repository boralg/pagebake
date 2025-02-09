use std::{collections::HashMap, fs, io, path::Path, rc::Rc};

use crate::{redirects::{Redirect, RedirectList, RedirectPageRenderer}, Router};

pub struct RenderMap {
    pub pages: HashMap<String, Box<dyn FnOnce() -> String>>,
    pub extra_files: HashMap<String, Box<dyn FnOnce() -> String>>,
}

pub struct RenderConfig {
    pub fallback_page_name: String,
    pub resolve_redirect_chains: bool,
    pub redirect_page_renderer: Option<RedirectPageRenderer>,
    pub redirect_list: Option<RedirectList>,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            fallback_page_name: "404".to_owned(),
            resolve_redirect_chains: false,
            redirect_page_renderer: Some(Redirect::base_redirect_page()),
            redirect_list: None,
        }
    }
}

impl Router {
    pub(crate) fn prepare_map(mut self, config: RenderConfig) -> RenderMap {
        if config.resolve_redirect_chains {
            self.redirects = self.resolve_redirects();
        }

        if let Some(renderer) = config.redirect_page_renderer {
            let renderer = Rc::new(renderer);

            for (source, target) in &self.redirects {
                let renderer = Rc::clone(&renderer);
                let target = target.to_owned();

                self.routes
                    .insert(source.to_owned(), Box::new(move || renderer(&target)));
            }
        }

        for (mut path, page) in self.fallbacks {
            if !path.ends_with("/") {
                path.push('/');
            }
            path.push_str(&config.fallback_page_name);

            if self.routes.contains_key(&path) {
                panic!("Overlap with fallback handler. Route `{path}` already exists");
            }

            self.routes.insert(path, page);
        }

        let mut extra_files = HashMap::<String, Box<dyn FnOnce() -> String>>::new();

        if let Some(renderer) = config.redirect_list {
            let redirects = self
                .redirects
                .into_iter()
                .map(|(source, target)| Redirect { source, target })
                .collect();

            extra_files.insert(
                renderer.file_name.to_owned(),
                Box::new(|| (renderer.content_renderer)(redirects)),
            );
        }

        RenderMap {
            pages: self.routes,
            extra_files,
        }
    }

    pub fn render(self, output_path: &Path, config: RenderConfig) -> io::Result<()> {
        let map = self.prepare_map(config);

        fs::create_dir_all(output_path)?;

        for (path, page) in map.pages {
            let page_path = match path.strip_prefix("/").unwrap() {
                "" => "index",
                path => path,
            };

            let mut export_path = output_path.to_path_buf();
            export_path.push(page_path);
            export_path.set_extension("html");

            fs::create_dir_all(export_path.parent().unwrap())?;
            fs::write(export_path, page())?;
        }

        for (path, page) in map.extra_files {
            let mut export_path = output_path.to_path_buf();
            export_path.push(path);

            fs::create_dir_all(export_path.parent().unwrap())?;
            fs::write(export_path, page())?;
        }

        Ok(())
    }
}
