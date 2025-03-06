# pagebake

`pagebake` is a simple, modular static site generator library. Inspired by [`axum`](https://crates.io/crates/axum), `pagebake` provides an intuitive API for defining routes, handling redirects, and rendering static HTML pages.

## Features

- **Routing and Rendering:**  Define custom routes that map to page-rendering functions. Use simple closures to generate HTML content.

- **Redirect Support:** Easily configure redirects that work with static hosting services, or out-of-the box via plain HTML.

- **Fallback Handlers:** Specify fallback pages for unmatched routes.

- **Router Composition:** Merge and nest routers to build modular and scalable site architectures.

- **Flexible Output Options:** Render your site directly to disk or generate an in-memory map of files.


## Installation

### Via Cargo

Add `pagebake` to your `Cargo.toml`:

```bash
cargo add pagebake
```

### From Source

Clone the repository:

```bash
git clone https://github.com/boralg/pagebake.git
cd pagebake
```

If you use Nix, simply activate the development shell:

```bash
nix develop
```


## Usage

### Defining Routes

Create a new router and register your routes with rendering functions or redirects:

```rust
use pagebake::render::RenderConfig;
use pagebake::{get, redirect, Router};

fn main() {
    let router = Router::new()
        .route("/", get(|| "<h1>Home</h1>".to_owned()))
        .route("/about", get(|| "<h1>About</h1>".to_owned()))
        .route("/old-home", redirect("/"))
        .fallback(|| "<h1>Not Found</h1>".to_owned());

    let config = RenderConfig::default();
    let _ = router.render(std::path::Path::new("./public"), config);
}
```

### Nesting Routers

Organize your site by nesting routers for different sections:

```rust
use pagebake::{get, redirect, Router};

fn main() {
    // Create a sub-router for blog-related pages.
    let blog_router = Router::new()
        .route("/", get(|| "<h1>Welcome to the Blog</h1>".to_owned()))
        .route("/post", get(|| "<h1>Blog Post</h1>".to_owned()))
        .route("/old", redirect("/"))
        .fallback(|| "<h1>Blog 404: Page Not Found</h1>".to_owned());

    // Nest the blog router under the "/blog" prefix.
    // This prepends "/blog" to all routes from the blog_router.
    let router = Router::new()
        .route("/", get(|| "<h1>Home</h1>".to_owned()))
        .nest("/blog", blog_router);

    // At this point, the following routes are available:
    // - "/" renders "<h1>Home</h1>"
    // - "/blog/" renders "<h1>Welcome to the Blog</h1>"
    // - "/blog/post" renders "<h1>Blog Post</h1>"
    // - "/blog/old" performs a redirect to "/blog/"
    // - The fallback route for unmatched blog paths would typically become a page at path "/blog/404"
}
```

### Redirects, Sitemaps and Custom Rendering

`pagebake` supports custom redirect page rendering. By default, a simple HTML page is generated that uses meta tags and JavaScript to perform the redirect. Custom renderers can also be configured.

For redirect list generation (e.g. for [Cloudflare Pages](https://pages.cloudflare.com/) or [Static Web Server](https://static-web-server.net/)), use the provided configurations in the `redirects` module.

The same applies to route lists, which can be used to generate sitemaps.

```rust
use pagebake::redirects::RedirectList;
use pagebake::render::RenderConfig;
use pagebake::routes::RouteList;
use pagebake::{get, redirect, Router};

fn main() {
    let router = Router::new()
        .route("/", get(|| "<h1>Home</h1>".to_owned()))
        .route("/old-page", redirect("/"));

    let redirect: Box<dyn Fn(&str) -> String> = Box::new(|target| {
        format!(
            r#"<!DOCTYPE HTML>
<script>
  window.location.href = "{0}";
</script>"#,
            target
        )
    });

    let config = RenderConfig {
        redirect_page_renderer: Some(redirect),
        redirect_lists: vec![RedirectList::for_cloudflare_pages()],
        route_lists: vec![RouteList::sitemap("http://localhost:8080".to_string())],
        ..Default::default()
    };

    // This will generate:
    // - An HTML page at "/old-page.html" using the custom redirect renderer.
    // - A "_redirects" file with the list of all redirects.
    // - A sitemap.xml containing all non-redirect routes.
    let _ = router.render(std::path::Path::new("./public"), config);
}
```

---

## Contributing

Contributions to `pagebake` are welcome! If you have suggestions, encounter issues, or want to contribute new features, please open an issue or submit a pull request.
