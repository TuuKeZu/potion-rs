# Potion-rs
> [!CAUTION]
> Should not be used in production, under development

Flask inspired general purpose fullstack web-framework with file-system based routing with support for server-side rendered template-based pages. For client side hydration and responsiveness potion includes build-in Typescript bundler *(as it is our philosophy that Typescript should be strictly used for simple DOM manipulation and)*

### Features
- [x] File-system based compile-time generated routing
    - [x] Robust state management between routers
    - [x] Support for accessing files both in router's own dir and in static folder.
- [x] typescriot support for post-render DOM-manipulation
    - [x] .ts files both in /static and /routing directories are automatically compiled and linked *(sourcemaps included)*
- [x] Optimised for fast rendering
    - [x] Minified generated HTML
    - [x] Minified .js bundles
    - [ ] Compressed HTML


## Example

`src/main.rs`
```rust
// Derive `potion::IntoContext` for global state
#[derive(Clone, potion::IntoContext)]
pub struct RouterContext {
    pub hb: Arc<Handlebars<'static>>,
    pub db: Pool<Postgres>
}

// Generate routing during compilation phase. All .rs files are automatically included during compilation
potion::routing!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    // Initialize file-system routing
    let (mut hb, static_router) = potion::initialize_routing(
        &std::env::var("POTION_PROJECT_DIR").expect("Tried to read misconfigured .env file"),
        true,
    )?;

    // Create postgres connection-pool
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("POSTGRES_URL").expect("Tried to read misconfigured .env file"))
        .await?;

    // Initialize context
    let context = Box::new(RsContext { hb: Arc::new(hb), db: pool.clone() });

    // Generate and server routing
    let routes = router(context)
        .or(static_router);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}
```

`src/routing/hello/index.rs`
```rust
pub fn initialize(router: potion::Router) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // Access global state
    let context = router.downcast::<RouterContext>();
    
    let routing = warp::path("hello")
        .and(warp::get())
        .map(|| "Hello World");

    routing
}
```