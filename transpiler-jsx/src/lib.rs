use transpilers::AssetTranspiler;
use transpilers::rquickjs::{Module, Ctx, Loaded, Result, Script};

#[derive(Default)]
pub struct JsxTranspiler {
    l: swc_tools::SwcTools
}

impl AssetTranspiler for JsxTranspiler {
    fn transpile<'js>(&mut self, ctx: Ctx<'js>, path: &str) -> Result<Module<'js, Loaded<Script>>> {
        let js_source = self.l.load_js(path);
        Module::new(ctx, path, js_source)
    }
}