use std::{collections::HashMap, rc::Rc};

use crate::{redirects::Redirect, RenderConfig, Router};

pub struct RenderMap {
    pub pages: HashMap<String, Box<dyn FnOnce() -> String>>,
    pub extra_files: HashMap<String, Box<dyn FnOnce() -> String>>,
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
}
