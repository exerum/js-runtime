use rquickjs::{Loaded, Result, Error, Module, Loader, Ctx};
use std::path::Path;
use crate::swc::loader::SwcLoader;
use super::module_specifier::ModuleSpecifier;
use std::collections::HashMap;
use crate::register;
use super::transpilers::{TKey, Transpilers, AssetTranspiler};

#[derive(Default)]
pub struct TypescriptTranspiler {
    l: SwcLoader
}

impl AssetTranspiler for TypescriptTranspiler {
    fn transpile<'js>(&mut self, ctx: Ctx<'js>, path: &str) -> Result<Module<'js, Loaded<rquickjs::Script>>> {
        let js_source = self.l.load_ts(path);
        let m = Module::new(ctx, path, js_source)?;
        Ok(m)
    }
}

#[derive(Default)]
pub struct JsTranspiler {
}

impl AssetTranspiler for JsTranspiler {
    fn transpile<'js>(&mut self, ctx: Ctx<'js>, path: &str) -> Result<Module<'js, Loaded<rquickjs::Script>>> {
        let js_source = std::fs::read_to_string(path).unwrap();
        Module::new(ctx, path, js_source)
    }
}

#[derive(Default)]
pub struct JsxTranspiler {
    l: SwcLoader
}

impl AssetTranspiler for JsxTranspiler {
    fn transpile<'js>(&mut self, ctx: Ctx<'js>, path: &str) -> Result<Module<'js, Loaded<rquickjs::Script>>> {
        let js_source = self.l.load_js(path);
        Module::new(ctx, path, js_source)
    }
}

pub struct ExerumLoader {
    cache_dir: String,
    transpilers: Transpilers,
    cache: HashMap<String, Vec<u8>>
}

impl ExerumLoader {
    pub fn new(cache_dir: &str) -> Self {
        let mut transpilers = Transpilers::default();
        register!(transpilers, "typescript", [.ts, .tsx], TypescriptTranspiler);
        register!(transpilers, "javascript_react", [.jsx], JsxTranspiler);
        register!(transpilers, "javascript", [.js], JsTranspiler);
        ExerumLoader {
            transpilers,
            cache_dir: cache_dir.to_owned(),
            cache: HashMap::new(),
        }
    }

    pub fn save_to_disk(&self, name: &str, data: &Vec<u8>) {
        let file_name = name.replace("/", "_");
        let path = Path::new(&self.cache_dir).join(file_name);
        std::fs::write(path, data).unwrap()
    }

}

impl Loader for ExerumLoader {

    fn load<'js>(&mut self, ctx: Ctx<'js>, name: &str) -> Result<Module<'js, Loaded>> {
        let ms = ModuleSpecifier::from(name);
        let name = ms.path();
        // if cach hit, retrieve from cache
        if let Some(serialized_module) = self.cache.get(name) {
            Ok(Module::read_object(ctx, serialized_module)?.into_loaded())
        } else {
            let m = if let Some(transpiler_name) = ms.transpiler() {
                // Pick transpiler by name
                let mut t = self.transpilers
                    .by_name(transpiler_name)
                    .ok_or(Error::new_loading(name))?;
                t.transpile(ctx, name)?
            } else if let Some(ext) = ms.extension() {
                // Pick transpiler by file extension
                let mut t = self.transpilers
                    .by_ext(ext)
                    .ok_or(Error::new_loading(name))?;
                t.transpile(ctx, name)?
            } else {
                // Default to javascript
                // TODO: change. Make a default key maybe.
                let js_source = std::fs::read_to_string(name).unwrap();
                let m = Module::new(ctx, name, js_source)?;
                m
            };
            let serialized = m.write_object(false)?;
            let loaded = m.into_loaded();
            self.save_to_disk(name, &serialized);
            self.cache.insert(name.to_owned(), serialized);
            Ok(loaded)
        }
    }

}