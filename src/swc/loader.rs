use std::{path::Path};
use swc_ecma_visit::FoldWith;
use swc_common::{
    self,
    GLOBALS,
    Globals,
    Mark,
    comments::SingleThreadedComments,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    SourceMap,
};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig, EsConfig};
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_transforms_typescript::strip;
use swc_ecma_transforms_react::react;
use swc_ecma_ast::Module;

#[derive(Default)]
pub struct SwcLoader {
}

impl SwcLoader {

    pub fn load_js(&self, path: &str) -> String {
        let cm: Lrc<SourceMap> = Default::default();
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
        let fm = cm
            .load_file(Path::new(path))
            .expect("failed to load source");
        // let comments = SingleThreadedComments::default();
        let lexer = Lexer::new(
            Syntax::Es(EsConfig {
                jsx: path.ends_with("jsx"),
                dynamic_import: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);

        for e in parser.take_errors() {
            e.into_diagnostic(&handler).emit();
        }

        let module = parser
            .parse_module()
            .map_err(|e| e.into_diagnostic(&handler).emit())
            .expect("failed to parse module.");
        // Ensure that we have enough parenthesis.
        let module = module.fold_with(&mut fixer(None));
            
        let module = GLOBALS.set(&Globals::default(), || {
            let top_level_mark = Mark::fresh(Mark::root());
            let module = module
                // Transform jsx
                .fold_with(&mut react::<SingleThreadedComments>(
                    cm.clone(),
                    None,
                    Default::default(),
                    top_level_mark,
                ));
            module
        });

        self.emit(&cm, &module)
    }

    pub fn load_ts(&mut self, path: &str) -> String {
        let cm: Lrc<SourceMap> = Default::default();
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
        let fm = cm
            .load_file(Path::new(path))
            .expect("failed to load source");
        // let comments = SingleThreadedComments::default();
        let lexer = Lexer::new(
            Syntax::Typescript(TsConfig {
                tsx: path.ends_with("tsx"),
                dynamic_import: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);

        for e in parser.take_errors() {
            e.into_diagnostic(&handler).emit();
        }

        let module = parser
            .parse_module()
            .map_err(|e| e.into_diagnostic(&handler).emit())
            .expect("failed to parse module.");
        // Ensure that we have enough parenthesis.
        let module = module.fold_with(&mut fixer(None));
            
        let module = GLOBALS.set(&Globals::default(), || {
            let top_level_mark = Mark::fresh(Mark::root());
            // Remove typescript types
            let module = module
                .fold_with(&mut strip())
                // Transform tsx
                .fold_with(&mut react::<SingleThreadedComments>(
                    cm.clone(),
                    None,
                    Default::default(),
                    top_level_mark,
                ));
            module
        });

        self.emit(&cm, &module)
    }

    fn emit(&self, cm: &Lrc<SourceMap>, module: &Module) -> String {
        let mut buf = vec![];
        {
            let mut emitter = Emitter {
                cfg: swc_ecma_codegen::Config { minify: false },
                cm: cm.clone(),
                comments: None,
                wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
            };
            emitter.emit_module(&module).unwrap();
        }
        String::from_utf8(buf).expect("non-utf8?")
    }
}
