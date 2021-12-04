use transpilers::AssetTranspiler;
use transpilers::rquickjs::{Module, Script, Ctx, Loaded, Result};

#[derive(Default)]
pub struct TypescriptTranspiler {
    l: swc_tools::SwcTools
}

impl AssetTranspiler for TypescriptTranspiler {
    fn transpile<'js>(&mut self, ctx: Ctx<'js>, path: &str) -> Result<Module<'js, Loaded<Script>>> {
        let js_source = self.l.load_ts(path);
        let m = Module::new(ctx, path, js_source)?;
        Ok(m)
    }
}