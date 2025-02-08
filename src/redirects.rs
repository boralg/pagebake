use std::collections::{HashMap, HashSet};

use crate::Router;

pub struct Redirect<'a> {
    pub source: &'a str,
    pub target: &'a str,
}

pub type RedirectPageRenderer = Box<dyn Fn(&str) -> String>;
pub type RedirectListRenderer = Box<dyn FnOnce(Vec<Redirect>) -> String>;

pub struct RedirectList {
    pub file_name: &'static str,
    pub content_renderer: RedirectListRenderer,
}

impl Redirect<'_> {
    pub fn base_redirect_page() -> RedirectPageRenderer {
        Box::new(|target_url| {
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
            window.location.href = "{0}";
        }})();
    </script>

    <p>Redirecting to <a href="{0}">{0}</a>...</p>
</body>
</html>"#,
                target_url
            )
        })
    }
}

impl RedirectList {
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
}

impl Router {
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
