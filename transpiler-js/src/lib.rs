use transpilers::AssetTranspiler;
use transpilers::rquickjs::{Module, Ctx, Loaded, Result, Script};

#[derive(Default)]
pub struct JsTranspiler {}

impl AssetTranspiler for JsTranspiler {
    fn transpile<'js>(&mut self, ctx: Ctx<'js>, path: &str) -> Result<Module<'js, Loaded<Script>>> {
        let js_source = std::fs::read_to_string(path).unwrap();
        Module::new(ctx, path, js_source)
    }
}