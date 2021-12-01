use rquickjs::Resolver;
use rquickjs::Loader;
use rquickjs::{BuiltinResolver, BuiltinLoader, FileResolver, ScriptLoader};

/// Use default loader and resolver
pub fn init_loader_and_resolver() -> (impl Loader, impl Resolver) {
    let resolver = (
        BuiltinResolver::default(),
        FileResolver::default()
            .with_path("./")
            .with_native(),
    );
    let loader = (
        BuiltinLoader::default(),
        ScriptLoader::default(),
    );
    (loader, resolver)
}