use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

pub struct Router {
    routes: HashMap<String, Box<dyn Fn() -> String>>,
    redirects: HashMap<String, String>,
    fallbacks: HashMap<String, Box<dyn Fn() -> String>>,
}

pub enum Response {
    Get(Box<dyn Fn() -> String>),
    Redirect(String),
}

pub fn get<R>(page: R) -> Response
where
    R: Fn() -> String + 'static,
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
        R: Fn() -> String + 'static,
    {
        if self.fallbacks.contains_key("/") {
            panic!("Overlapping method route. Fallback handler already exists");
        }

        self.fallbacks.insert("/".to_owned(), Box::new(page));
        self
    }

    // pub fn merge(mut self, router: Router) -> Self {
    //     for (source, target) in router.redirects {
    //         self.redirects.insert(
    //             source,
    //             ,
    //         );
    //     }
    // }

    pub fn render(mut self, output_path: &Path) -> io::Result<()> {
        fs::create_dir_all(output_path)?;

        for (source, target) in self.redirects {
            self.routes.insert(
                source,
                Box::new(move || Self::render_redirect_page(&target)),
            );
        }

        for (mut path, page) in self.fallbacks {
            path.push_str("404");

            if self.routes.contains_key(&path) {
                panic!("Overlap with fallback handler. Route `{path}` already exists");
            }

            self.routes.insert(path, page);
        }

        for (path, page) in &self.routes {
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
