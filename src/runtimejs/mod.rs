pub mod resolver;
pub mod runtime;
pub mod loader;

use self::loader::ExerumLoader;
use self::resolver::ExerumResolver;
use rquickjs::{Resolver, Loader, FileResolver, ScriptLoader};

pub fn init_loader_and_resolver(source_root: &str) -> (impl Loader, impl Resolver) {
    let resolver = (
        ExerumResolver::new(source_root),
    );
    let loader = (
        ExerumLoader::new(),
    );
    (loader, resolver)
}

/// Returns default loader and resolver
pub fn init_default_loader_and_resolver() -> (impl Loader, impl Resolver) {
    let resolver = (
        FileResolver::default()
            .with_path("./")
    );
    let loader = (
        ScriptLoader::default(),
    );
    (loader, resolver)
}