use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
};

pub struct Router {
    routes: HashMap<String, Box<dyn FnOnce() -> String>>,
    redirects: HashMap<String, String>,
    fallbacks: HashMap<String, Box<dyn FnOnce() -> String>>,
}

pub struct RenderConfig {
    fallback_page_name: String,
    resolve_redirect_chains: bool,
    create_redirect_pages: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            fallback_page_name: "404".to_owned(),
            resolve_redirect_chains: true,
            create_redirect_pages: true,
        }
    }
}

pub enum Response {
    Get(Box<dyn FnOnce() -> String>),
    Redirect(String),
}

pub fn get<R>(page: R) -> Response
where
    R: FnOnce() -> String + 'static,
{
    Response::Get(Box::new(page))
}

pub fn redirect(path: &str) -> Response {
    Response::Redirect(path.to_owned())
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            redirects: HashMap::new(),
            fallbacks: HashMap::new(),
        }
    }

    pub fn route(mut self, path: &str, response: Response) -> Self {
        fn validate_path(path: &str) {
            if path.is_empty() {
                panic!("Paths must start with a `/`. Use \"/\" for root routes");
            } else if !path.starts_with('/') {
                panic!("Paths must start with a `/`");
            }
        }

        validate_path(path);

        if self.routes.contains_key(path) || self.redirects.contains_key(path) {
            panic!("Overlapping method route. Handler for `{path}` already exists");
        }

        match response {
            Response::Get(page) => {
                self.routes.insert(path.to_owned(), page);
            }
            Response::Redirect(redirect_path) => {
                validate_path(&redirect_path);
                self.redirects.insert(path.to_owned(), redirect_path);
            }
        };

        self
    }

    pub fn fallback<R>(mut self, page: R) -> Self
    where
        R: FnOnce() -> String + 'static,
    {
        if self.fallbacks.contains_key("/") {
            panic!("Overlapping method route. Fallback handler already exists");
        }

        self.fallbacks.insert("/".to_owned(), Box::new(page));
        self
    }

    pub fn merge(mut self, router: Router) -> Self {
        for (source, target) in router.redirects {
            if self.redirects.contains_key(&source) {
                panic!("Overlapping method route. Redirect handler for `{source}` already exists");
            }
            self.redirects.insert(source, target);
        }

        for (path, page) in router.routes {
            if self.routes.contains_key(&path) {
                panic!("Overlapping method route. Handler for `{path}` already exists");
            }
            self.routes.insert(path, page);
        }

        for (path, page) in router.fallbacks {
            if self.fallbacks.contains_key(&path) {
                panic!("Overlapping method route. Fallback handler for `{path}` already exists");
            }
            self.fallbacks.insert(path, page);
        }

        self
    }

    pub fn nest(self, prefix: &str, router: Router) -> Self {
        let prefix = if prefix == "/" {
            "".to_owned()
        } else {
            prefix.trim_end_matches('/').to_owned()
        };

        let mut router = router;

        router.redirects = router
            .redirects
            .into_iter()
            .map(|(source, target)| (format!("{prefix}{source}"), format!("{prefix}{target}")))
            .collect();

        router.routes = router
            .routes
            .into_iter()
            .map(|(path, page)| (format!("{prefix}{path}"), page))
            .collect();

        router.fallbacks = router
            .fallbacks
            .into_iter()
            .map(|(path, page)| (format!("{prefix}{path}"), page))
            .collect();

        self.merge(router)
    }

    pub fn render(mut self, output_path: &Path, config: RenderConfig) -> io::Result<()> {
        fs::create_dir_all(output_path)?;

        if config.resolve_redirect_chains {
            self.redirects = self.resolve_redirects();
        }

        if config.create_redirect_pages {
            for (source, target) in self.redirects {
                self.routes.insert(
                    source,
                    Box::new(move || Self::render_redirect_page(&target)),
                );
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

        for (path, page) in self.routes {
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

        Ok(())
    }

    fn resolve_redirects(&self) -> HashMap<String, String> {
        let mut resolved = HashMap::<String, String>::new();

        for (source, target) in &self.redirects {
            let mut visited = HashSet::<&String>::new();
            visited.insert(&source);

            let mut final_target = target;

            while let Some(next_target) = self.redirects.get(final_target) {
                if visited.contains(next_target) {
                    panic!("Cycle in redirects. Page `{next_target}` is both a source and target");
                }

                visited.insert(final_target);
                final_target = next_target;
            }

            resolved.insert(source.to_owned(), final_target.to_owned());
        }

        resolved
    }

    fn render_redirect_page(target_url: &str) -> String {
        format!(
            r#"<!DOCTYPE HTML>
<meta charset="UTF-8">
<meta http-equiv="refresh" content="0; url={0}">
 
<script>
  window.location.href = "{0}";
</script>
 
<title>Page Redirection</title>

Redirecting to <a href="{0}">{0}</a>..."#,
            target_url
        )
    }
}
