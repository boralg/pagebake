use std::collections::{HashMap, HashSet};

use crate::Router;

pub struct Redirect<'a> {
    pub source: &'a str,
    pub target: &'a str,
}

type RedirectListRenderer = Box<dyn FnOnce(Vec<Redirect>) -> String>;

pub struct RedirectList {
    pub file_name: &'static str,
    pub content_renderer: RedirectListRenderer,
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
                    panic!("Cycle in redirects. Page `{next_target}` is both a source and target");
                }

                visited.insert(final_target);
                final_target = next_target;
            }

            resolved.insert(source.to_owned(), final_target.to_owned());
        }

        resolved
    }

    pub(crate) fn render_redirect_page(target_url: &str) -> String {
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
