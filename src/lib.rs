use std::collections::HashMap;

pub mod redirects;
pub mod render;

pub struct Router {
    routes: HashMap<String, Box<dyn FnOnce() -> String>>,
    redirects: HashMap<String, String>,
    fallbacks: HashMap<String, Box<dyn FnOnce() -> String>>,
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
}
