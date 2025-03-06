use std::collections::{HashMap, HashSet};

use crate::Router;

/// Represents a redirection from a source path to a target path.
#[derive(Debug, Clone)]
pub struct Redirect {
    pub source: String,
    pub target: String,
}

/// A function that, given a target path, renders a page that redirects to it.
pub type RedirectPageRenderer = Box<dyn Fn(&str) -> String>;

/// A function that renders a list of redirects, given a vector of `Redirect` objects.
/// Redirect lists can be utilized by static hosting services.
pub type RedirectListRenderer = Box<dyn FnOnce(Vec<Redirect>) -> String>;

/// Configuration for generating a redirect list file.
pub struct RedirectList {
    /// The name of the output file.
    pub file_name: &'static str,
    /// Function that takes a list of `Redirect` objects and returns the redirect list's content.
    pub content_renderer: RedirectListRenderer,
}

impl Redirect {
    /// Returns a default redirect page renderer.
    ///
    /// This renderer produces an HTML page that immediately redirects the user to the specified target path.
    /// The output includes meta tags and JavaScript to facilitate the redirect.
    /// In case both fail, a clickable link is included that points to the target path.
    pub fn base_redirect_page() -> RedirectPageRenderer {
        Box::new(|target| {
            format!(
                r#"<!DOCTYPE HTML>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="refresh" content="0; url={0}">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Page Redirection</title>
</head>
<body>
    <script>
        (function() {{
            window.location.replace("{0}");
        }})();
    </script>

    <p>Redirecting to <a href="{0}">{0}</a>...</p>
</body>
</html>"#,
                target
            )
        })
    }
}

impl RedirectList {
    /// Creates a `RedirectList` configuration for [Cloudflare Pages](https://pages.cloudflare.com/).
    ///
    /// The generated file will be named `_redirects` and contain the list of redirects in a format
    /// compatible with Cloudflare Pages.
    pub fn for_cloudflare_pages() -> Self {
        RedirectList {
            file_name: "_redirects",
            content_renderer: Box::new(|redirects: Vec<Redirect>| {
                redirects
                    .iter()
                    .map(|r| format!("{} {}", r.source, r.target))
                    .collect::<Vec<String>>()
                    .join("\n")
            }),
        }
    }

    /// Creates a `RedirectList` configuration for [Static Web Server](https://static-web-server.net/).
    ///
    /// The generated file will be named `config.toml` and contain the list of redirects as an array of tables.
    pub fn for_static_web_server() -> Self {
        RedirectList {
            file_name: "config.toml",
            content_renderer: Box::new(|redirects: Vec<Redirect>| {
                let mut content = String::from("[advanced]\n\n");

                content.push_str(
                    &redirects
                        .iter()
                        .map(|r| {
                            format!(
                                "[[advanced.redirects]]\nsource = \"{}\"\ndestination = \"{}\"\nkind = 302",
                                r.source, r.target
                            )
                        })
                        .collect::<Vec<String>>()
                        .join("\n\n"),
                );

                content
            }),
        }
    }
}

impl Router {
    /// Resolves chained redirects into their final target path.
    ///
    /// This method traverses redirect chains to avoid cycles and ensure that each source path maps
    /// to the ultimate target path.
    ///
    /// # Panics
    ///
    /// Panics if a cycle is detected in the redirect chain.
    pub(crate) fn resolve_redirects(&self) -> HashMap<String, String> {
        let mut resolved = HashMap::<String, String>::new();

        for (source, target) in &self.redirects {
            let mut visited = HashSet::<&String>::new();
            visited.insert(&source);

            let mut final_target = target;

            while let Some(next_target) = self.redirects.get(final_target) {
                if visited.contains(next_target) {
                    panic!("Cycle in redirects starting at `{next_target}`");
                }

                visited.insert(final_target);
                final_target = next_target;
            }

            resolved.insert(source.to_owned(), final_target.to_owned());
        }

        resolved
    }
}
