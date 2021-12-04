use crate::cache::{ModuleCache, ModuleId};
use super::module_specifier::ModuleSpecifier;
use rquickjs::{generic_loader, Ctx, Error, Loaded, Loader, Module, Result, Script};
use transpilers::Transpilers;

generic_loader! {
    ExerumLoader: Script,
}

pub struct ExerumLoader {
    cache: Box<dyn ModuleCache<ModuleId, Vec<u8>>>,
    transpilers: Transpilers,
}

impl ExerumLoader {
    pub fn new(cache: Box<dyn ModuleCache<ModuleId, Vec<u8>>>, transpilers: Transpilers) -> Self {
        ExerumLoader {
            transpilers,
            cache
        }
    }
}

impl Loader<Script> for ExerumLoader {
    fn load<'js>(&mut self, ctx: Ctx<'js>, name: &str) -> Result<Module<'js, Loaded<Script>>> {
        let ms = ModuleSpecifier::from(name);
        let name = ms.path();
        let name_owned = name.to_owned();
        // if cach hit, retrieve from cache
        if let Some(serialized_module) = self.cache.get(&name_owned) {
            Ok(Module::read_object(ctx, serialized_module)?)
        } else {
            let m = if let Some(transpiler_name) = ms.transpiler() {
                // Pick transpiler by name
                let mut t = self
                    .transpilers
                    .by_name(transpiler_name)
                    .ok_or(Error::new_loading(name))?;
                t.transpile(ctx, name)?
            } else if let Some(ext) = ms.extension() {
                // Pick transpiler by file extension
                let mut t = self
                    .transpilers
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
            self.cache.insert(name_owned, serialized);
            Ok(m)
        }
    }
}

#[cfg(test)]
mod tests {
    use rquickjs::{Func, Function, Promise, Tokio, Module};
    use transpiler_js::JsTranspiler;
    use crate::cache::NoCache;
    use transpiler_jsx::JsxTranspiler;
    use transpiler_typescript::TypescriptTranspiler;
    use transpilers::{Transpilers, register, AssetTranspiler, TKey};
    #[test]
    fn test_resolver_loader() {
        fn print(msg: String) {
            println!("{}", msg);
        }
        let mut transpilers = Transpilers::default();
        register!(transpilers, "typescript", [.ts, .tsx], TypescriptTranspiler);
        register!(transpilers, "javascript_react", [.jsx], JsxTranspiler);
        register!(transpilers, "javascript", [.js], JsTranspiler);
        let resolver = crate::resolver::ExerumResolver::new("./test_data/");
        let loader = crate::loader::ExerumLoader::new(Box::new(NoCache{}), transpilers);

        let tokio_rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .enable_time()
            .build()
            .unwrap();

        let local_set = tokio::task::LocalSet::new();
        let fut = local_set.run_until(async move {
            let rt = crate::runtime::JsRuntime::new(loader, resolver);
            let context = rt.context();
            rt.rt().spawn_executor(Tokio);
            context.with(|ctx| {
                let global = ctx.globals();
                global.set("print", Func::new("print", print)).unwrap();
            });
            let contents = std::fs::read_to_string("test_data/src/main.tsx").unwrap();
            let promise: Promise<String> = context.with(|ctx| {
                let m = Module::new(ctx, "src/main.tsx", contents.as_bytes().to_vec()).unwrap();
                let loaded = m.eval().unwrap();
                let main: Function = loaded.get("main").unwrap();
                main.call(()).unwrap()
            });
            let res = promise.await.unwrap();
            assert_eq!(res, "Done.reactAB");
            rt.rt().idle().await;
        });
        tokio_rt.block_on(fut);
    }

    #[test]
    fn test_transpilers() {
        let mut transpilers = Transpilers::default();
        register!(transpilers, "typescript", [.ts, .tsx], TypescriptTranspiler);
        register!(transpilers, "javascript_react", [.jsx], JsxTranspiler);
        register!(transpilers, "javascript", [.js], JsTranspiler);
        transpilers.by_ext("js").unwrap();
    }
}
