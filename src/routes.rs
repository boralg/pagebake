/// A function that renders a list of routes, given a vector of routes.
/// Route lists can be used to generate sitemaps.
pub type RouteListRenderer = Box<dyn FnOnce(Vec<String>) -> String>;

/// Configuration for generating a route list file.
pub struct RouteList {
    /// The name of the output file.
    pub file_name: &'static str,
    /// Function that takes a list of routes and returns the route list's content.
    pub content_renderer: RouteListRenderer,
    /// Whether to include redirect endpoints to the routes.
    pub include_redirects: bool,
}

impl RouteList {
    /// Creates a `RouteList` configuration for sitemaps.
    ///
    /// The generated file will be named `sitemap.xml` and contain the all non-redirect routes arranged as a sitemap.
    pub fn sitemap(origin_url: String) -> Self {
        RouteList {
            file_name: "sitemap.xml",
            content_renderer: Box::new(move |routes: Vec<String>| {
                let mut content = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
                content
                    .push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

                content.push_str(
                    &routes
                        .iter()
                        .map(|r| format!("  <url>\n    <loc>{}{}</loc>\n  </url>", &origin_url, r))
                        .collect::<Vec<String>>()
                        .join("\n"),
                );

                content.push_str("\n</urlset>");
                content
            }),
            include_redirects: false,
        }
    }
}
