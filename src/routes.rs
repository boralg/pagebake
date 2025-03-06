pub type RouteListRenderer = Box<dyn FnOnce(Vec<String>) -> String>;

pub struct RouteList {
    pub file_name: &'static str,
    pub content_renderer: RouteListRenderer,
    pub include_redirects: bool,
}

impl RouteList {
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
