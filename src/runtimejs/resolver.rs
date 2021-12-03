use rquickjs::{Ctx, Result, Resolver};
use std::collections::HashMap;
use relative_path::{RelativePathBuf, RelativePath};
use std::path::PathBuf;
use super::module_specifier::ModuleSpecifier;
use rquickjs::Error;

#[derive(Debug, Default)]
pub struct ExerumResolver {
    project_root: PathBuf,
    aliases: HashMap<String, String>
}

impl ExerumResolver {

    pub fn new(project_root: &str) -> Self {
        ExerumResolver {
            project_root: PathBuf::from(project_root),
            aliases: HashMap::new()
        }
    }

    fn resolve_alias(&self, name: &str, relative_path: &RelativePath) -> Option<RelativePathBuf> {
        self.aliases.get(&name.to_owned())
            .map(|path| {
                relative_path.join_normalized(&path)
            })
    }

    fn resolve_node_modules(base: &RelativePathBuf, target: &str) -> Option<RelativePathBuf> {
        let mut base = base.parent();
        while let Some(dir) = base {
            let node_modules = dir.join("node_modules");
            if node_modules.to_path(".").is_dir() {
                let path = node_modules.join(target);
                if path.to_path(".").exists() {
                    return Some(path);
                }
            }
            base = dir.parent();
        }
        None
    }

    #[inline]
    fn resolve_internal(&mut self, base: &str, name: &str) -> Result<RelativePathBuf> {
        // Strip the transpiler name if any
        let ms = ModuleSpecifier::from(name);
        let name = ms.path();
        let project_root = RelativePath::new(self.project_root.to_str().unwrap());
        let base_buf = project_root.join_normalized(base);
        if name.starts_with('.') {
            // Resolve relative to file's parent only
            let target = if let Some(parent_dir) = base_buf.parent() {
                parent_dir.join_normalized(name)
            } else {
                RelativePathBuf::from(name)
            };
            if target.to_path(".").exists() {
                return Ok(target);
            }
        } else {
            // Current folder first
            if let Some(parent_dir) = base_buf.parent() {
                let target = parent_dir.join_normalized(name);
                if target.to_path(".").exists() {
                    return Ok(target);
                }
            }
            // Node modules
            let in_node_modules = Self::resolve_node_modules(&base_buf, name);
            if let Some(p) = in_node_modules {
                return Ok(p);
            }
            // Project root
            let target = project_root.join_normalized(name);
            if target.to_path(".").exists() {
                return Ok(target)
            }
        };
        // Resolve aliases
        if let Some(alias) = self.resolve_alias(name, &project_root) {
            return Ok(alias)
        }
        return Err(Error::new_resolving(base, name));
    }
}

impl Resolver for ExerumResolver {
    fn resolve<'js>(&mut self, _ctx: Ctx<'js>, base: &str, name: &str) -> Result<String> {
        self.resolve_internal(base, name)
            .map(|p| p.to_path(".").to_str().unwrap().to_owned())
    }
}

#[test]
fn test_resolver_relative_to_current_file() {
    let mut resolver = ExerumResolver::new("test");
    // import * as React from './react.js' // in src/main.tsx
    let resolved = resolver.resolve_internal("src/main.tsx", "./react.js").unwrap();
    assert_eq!(resolved, "test/src/react.js");
}

#[test]
fn test_resolver_up_dir() {
    let mut resolver = ExerumResolver::new("test");
    // import * as React from '../src/react.js' // in src/main.tsx
    let resolved = resolver.resolve_internal("src/main.tsx", "../src/react.js").unwrap();
    assert_eq!(resolved, "test/src/react.js");
}

// Note: rquickjs treats "./test/src/main.js" and "test/src/main.js" as different modules

// #[test]
// fn test_resolver_relative_to_project_root() {
//     // This test will not pass because cargo wasi does not support arguments to wasm runner (wasmtime)
//     // https://github.com/bytecodealliance/cargo-wasi/blob/main/src/lib.rs#L200
//     let mut resolver = ExerumResolver::new("test");
//     // import * as React from 'src/react.js' // in src/main.tsx
//     let resolved = resolver.resolve_internal("src/main.tsx", "src/react.js").unwrap();
//     assert_eq!(resolved, "test/src/react.js");
// }

// #[test]
// fn test_resolver_node_modules() {
//     // This test will not pass because cargo wasi does not support arguments to wasm runner (wasmtime)
//     // https://github.com/bytecodealliance/cargo-wasi/blob/main/src/lib.rs#L200
//     let mut resolver = ExerumResolver::new("test");
//     // import * as React from 'react/umd/react.js' // in src/main.tsx
//     let resolved = resolver.resolve_internal("src/main.tsx", "react/umd/react.js").unwrap();
//     assert_eq!(resolved, "test/node_modules/react/umd/react.js");
// }