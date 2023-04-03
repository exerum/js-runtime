use std::path::Path;
use transpilers::AssetTranspiler;
use transpilers::rquickjs::{Module as QJSModule, Ctx, Loaded, Result, Script};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, EsConfig};
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_transforms_react::react;
use swc_ecma_visit::FoldWith;
use swc_ecma_ast::{EsVersion, Module};
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

#[derive(Default)]
pub struct JsxTranspiler {}

impl AssetTranspiler for JsxTranspiler {
    fn transpile<'js>(&mut self, ctx: Ctx<'js>, path: &str) -> Result<QJSModule<'js, Loaded<Script>>> {
        let cm: Lrc<SourceMap> = Default::default();
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
        let fm = cm
            .load_file(Path::new(path))
            .expect("failed to load source");
        // let comments = SingleThreadedComments::default();
        let lexer = Lexer::new(
            Syntax::Es(EsConfig {
                jsx: path.ends_with("jsx"),
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

        let js_source = emit(&cm, &module);
        QJSModule::new(ctx, path, js_source)
    }
}

fn emit(cm: &Lrc<SourceMap>, module: &Module) -> String {
    let mut buf = vec![];
    {
        let mut emitter = Emitter {
            cfg: swc_ecma_codegen::Config {
                ascii_only: false,
                omit_last_semi: true,
                target: EsVersion::Es2020,
                minify: false
            },
            cm: cm.clone(),
            comments: None,
            wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
        };
        emitter.emit_module(&module).unwrap();
    }
    String::from_utf8(buf).expect("non-utf8?")
}