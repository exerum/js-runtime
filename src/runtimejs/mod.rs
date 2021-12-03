pub mod resolver;
pub mod runtime;
pub mod loader;
pub mod module_specifier;
pub(crate) mod transpilers;

use self::loader::ExerumLoader;
use self::resolver::ExerumResolver;
use rquickjs::{Resolver, Loader, FileResolver, ScriptLoader};

pub fn init_loader_and_resolver(source_root: &str) -> (impl Loader, impl Resolver) {
    std::fs::create_dir("./.exerum/").unwrap();
    let resolver = (
        ExerumResolver::new(source_root),
    );
    let loader = (
        ExerumLoader::new("./.exerum/"),
    );
    (loader, resolver)
}

/// Returns default loader and resolver
pub fn _init_default_loader_and_resolver() -> (impl Loader, impl Resolver) {
    let resolver = FileResolver::default()
        .with_path("./");
    let loader = ScriptLoader::default();
    (loader, resolver)
}