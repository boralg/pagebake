use std::collections::HashMap;

pub mod redirects;
pub mod render;

/// Router type to map paths to pages.
pub struct Router {
    routes: HashMap<String, Box<dyn FnOnce() -> String>>,
    redirects: HashMap<String, String>,
    fallbacks: HashMap<String, Box<dyn FnOnce() -> String>>,
}

/// Possible responses that route paths can be mapped to.
pub enum Response {
    /// GET response wrapping the provided page rendering function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// Response::Get(Box::new(|| "<h1>Hello, world!</h1>".to_owned()));
    /// ```
    Get(Box<dyn FnOnce() -> String>),
    /// Redirect response that points to another path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// Response::Redirect("/home".to_owned());
    /// ```
    Redirect(String),
}

/// Wraps a page rendering function into a GET response.
///
/// # Examples
///
/// ```rust
/// pagebake::get(|| "<h1>Hello, world!</h1>".to_owned());
/// ```
pub fn get<R>(page: R) -> Response
where
    R: FnOnce() -> String + 'static,
{
    Response::Get(Box::new(page))
}

/// Creates a redirect response to the specified path.
///
/// # Examples
///
/// ```rust
/// pagebake::redirect("/home");
/// ```
pub fn redirect(path: &str) -> Response {
    Response::Redirect(path.to_owned())
}

impl Router {
    /// Creates a new, empty `Router`.
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            redirects: HashMap::new(),
            fallbacks: HashMap::new(),
        }
    }

    /// Adds a new route to the `Router`.
    ///
    /// Depending on the `response` variant, the route will either render a page or perform a redirect.
    /// The provided `path` must start with a `/` and must not conflict with existing pages or redirects.
    ///
    /// # Panics
    ///
    /// Panics if the path is invalid or if a handler for the specified path already exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pagebake::{Router, Response, get, redirect};
    ///
    /// let router = Router::new()
    ///     .route("/", get(|| "<h1>Home</h1>".to_owned()))
    ///     .route("/about", get(|| "<h1>About</h1>".to_owned()))
    ///     .route("/old-home", redirect("/"));
    /// ```
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

    /// Sets a fallback handler for unmatched routes.
    ///
    /// The fallback page is used when no other route matches the incoming path.
    ///
    /// # Panics
    ///
    /// Panics if a fallback handler is already set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pagebake::{Router, get};
    ///
    /// let router = Router::new()
    ///     .route("/", get(|| "<h1>Home</h1>".to_owned()))
    ///     .fallback(|| "<h1>404 Not Found</h1>".to_owned());
    /// ```
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

    /// Merges another `Router` into the current one.
    ///
    /// This method combines routes, redirects, and fallback handlers from another router.
    /// Any overlapping routes will cause a panic.
    ///
    /// # Panics
    ///
    /// Panics if there is an overlapping route, redirect, or fallback.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pagebake::{Router, get};
    ///
    /// let router1 = Router::new().route("/", get(|| "<h1>Home</h1>".to_owned()));
    /// let router2 = Router::new().route("/blog", get(|| "<h1>Blog</h1>".to_owned()));
    ///
    /// let merged_router = router1.merge(router2);
    /// ```
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

    /// Nests a router under a specified path prefix.
    ///
    /// All routes, redirects, and fallback handlers from the nested router will have the prefix prepended.
    /// A prefix of "/" is equivalent to no prefix.
    ///
    /// # Panics
    ///
    /// Panics if any resulting route conflicts with existing routes.
    ///
    /// # Examples
    ///
    /// This example demonstrates nesting a blog sub-router under the "/blog" prefix.
    /// All routes defined in the blog router will be available under the "/blog" URL segment.
    ///
    /// ```rust
    /// use pagebake::{Router, get, redirect};
    ///
    /// // Create a sub-router for blog-related pages.
    /// let blog_router = Router::new()
    ///     .route("/", get(|| "<h1>Welcome to the Blog</h1>".to_owned()))
    ///     .route("/post", get(|| "<h1>Blog Post</h1>".to_owned()))
    ///     .route("/old", redirect("/"))
    ///     .fallback(|| "<h1>Blog 404: Page Not Found</h1>".to_owned());
    ///
    /// // Nest the blog router under the "/blog" prefix.
    /// // This prepends "/blog" to all routes from the blog_router.
    /// let router = Router::new()
    ///     .route("/", get(|| "<h1>Home</h1>".to_owned()))
    ///     .nest("/blog", blog_router);
    ///
    /// // At this point, the following routes are available:
    /// // - "/" renders "<h1>Home</h1>"
    /// // - "/blog/" renders "<h1>Welcome to the Blog</h1>"
    /// // - "/blog/post" renders "<h1>Blog Post</h1>"
    /// // - "/blog/old" performs a redirect to "/blog/"
    /// // - The fallback route for unmatched blog paths would typically become a page at path "/blog/404"
    /// ```
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
