use rquickjs::{Loaded, Result, Module, Loader, Ctx};
use std::path::Path;
use crate::swc::compiler::TSCompiler;

pub struct ExerumLoader {
    compiler: TSCompiler
}

impl ExerumLoader {
    pub fn new() -> Self {
        ExerumLoader {
            compiler: TSCompiler::new()
        }
    }
}

impl Loader for ExerumLoader {

    fn load<'js>(&mut self, ctx: Ctx<'js>, name: &str) -> Result<Module<'js, Loaded>> {
        let js_source = self.compiler.compile_file(Path::new(name));
        Ok(Module::new(ctx, name, js_source)?.into_loaded())
    }

}