use std::{collections::HashMap, fs, io, path::Path, rc::Rc};

use crate::{
    redirects::{Redirect, RedirectList, RedirectPageRenderer},
    Router,
};

/// Mapping of route paths to rendering functions.
struct RenderMap {
    /// Maps route paths to functions that return HTML content.
    pages: HashMap<String, Box<dyn FnOnce() -> String>>,
    /// Maps additional file paths (e.g. redirect lists) to their content generators.
    extra_files: HashMap<String, Box<dyn FnOnce() -> String>>,
}

/// Mapping of route paths to rendered outputs.
pub struct OutputMap {
    /// Maps route paths to their rendered HTML content.
    pub pages: HashMap<String, String>,
    /// Maps additional file paths to their rendered content.
    pub extra_files: HashMap<String, String>,
}

/// Configuration options for the rendering process.
pub struct RenderConfig {
    /// The name of fallback pages.
    pub fallback_page_name: String,
    /// When true, chains of redirects will be resolved to their final target.
    pub resolve_redirect_chains: bool,
    /// Optional custom renderer for redirect pages.
    /// When `None`, no redirect pages are included in the output.
    pub redirect_page_renderer: Option<RedirectPageRenderer>,
    /// Optional configuration for generating a file containing redirect mappings.
    /// When `None`, no redirect list is included in the output.
    pub redirect_list: Option<RedirectList>,
}

impl Default for RenderConfig {
    /// Provides default rendering configuration.
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
    /// Prepares a `RenderMap` based on registered routes and a `Router` configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The rendering configuration options.
    fn prepare_map(mut self, config: RenderConfig) -> RenderMap {
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

    /// Renders the site to the specified output directory.
    ///
    /// This function creates necessary directories and writes rendered pages and any additional files (e.g. redirect lists) to disk.
    ///
    /// # Arguments
    ///
    /// * `output_path` - The directory where rendered files will be written.
    /// * `config` - The rendering configuration options.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if file operations fail.
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

        for (path, file) in map.extra_files {
            let mut export_path = output_path.to_path_buf();
            export_path.push(path);

            fs::create_dir_all(export_path.parent().unwrap())?;
            fs::write(export_path, file())?;
        }

        Ok(())
    }

    /// Renders the site into an in-memory map.
    ///
    /// Returns an `OutputMap` where:
    /// - Keys represent the file paths (relative to the site root)
    /// - Values are the rendered content for each HTML page and and any additional files (e.g. redirect lists).
    pub fn render_to_map(self, config: RenderConfig) -> OutputMap {
        let map = self.prepare_map(config);

        OutputMap {
            pages: map
                .pages
                .into_iter()
                .map(|(path, page)| (path, page()))
                .collect(),
            extra_files: map
                .extra_files
                .into_iter()
                .map(|(path, file)| (path, file()))
                .collect(),
        }
    }
}
