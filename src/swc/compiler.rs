use anyhow::Error;
use std::path::PathBuf;
use std::{path::Path, sync::Arc};
use swc::config::Config;
use swc::config::JscConfig;
use swc::{self, config::Options};
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::EsConfig;
use swc_ecma_parser::{Syntax, TsConfig};

fn ts_options() -> Options {
    Options {
        config: Config {
            jsc: JscConfig {
                target: Some(EsVersion::Es2020),
                syntax: Some(Syntax::Typescript(TsConfig {
                    ..Default::default()
                })),
                ..Default::default()
            },
            module: None,
            ..Default::default()
        },
        swcrc: false,
        is_module: true,
        ..Default::default()
    }
}

fn js_options() -> Options {
    Options {
        config: Config {
            jsc: JscConfig {
                target: Some(EsVersion::Es2020),
                syntax: Some(Syntax::Es(EsConfig {
                    ..Default::default()
                })),
                ..Default::default()
            },
            module: None,
            ..Default::default()
        },
        swcrc: false,
        is_module: true,
        ..Default::default()
    }
}

fn swc_options_by_file_ext(file_ext: &str) -> Options {
    match file_ext {
        "ts" | "tsx" => ts_options(),
        "js" | "jsx" => js_options(),
        _ => {
            // TODO: support imported assets
            unimplemented!()
        }
    }
}

pub struct TSCompiler {
    sm: Arc<SourceMap>,
    handler: Arc<Handler>,
    c: swc::Compiler,
}

impl TSCompiler {
    pub fn new() -> Self {
        let sm = Arc::<SourceMap>::default();
        TSCompiler {
            sm: sm.clone(),
            handler: Arc::new(Handler::with_tty_emitter(
                ColorConfig::Auto,
                true,
                false,
                Some(sm.clone()),
            )),
            c: swc::Compiler::new(sm),
        }
    }

    pub fn compile_file(&mut self, path: &Path) -> String {
        let file_ext = path.extension().unwrap().to_str().unwrap();
        let opts = swc_options_by_file_ext(file_ext);
        let fm = self.sm.load_file(path).expect("failed to load file");
        self.c
            .process_js_file(fm, &self.handler, &opts)
            .map(|transformed_output| transformed_output.code)
            .unwrap()
    }
}

/// Mirrors the folder structure in src_dir to out_dir
fn write_js_file(src_dir: &PathBuf, out_dir: &PathBuf, original_path: &Path, contents: &[u8]) {
    let mut output_path = out_dir.clone();
    let relative_path = original_path
        .strip_prefix(Path::new(src_dir))
        .unwrap()
        .parent()
        .unwrap();
    output_path.push(relative_path);
    if !output_path.exists() {
        std::fs::create_dir_all(&output_path).unwrap();
    }
    // Make file name
    let out_file_name = original_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
        + ".js";
    output_path.push(out_file_name);
    std::fs::write(output_path, contents).unwrap();
}

// src_dir - src directory with all source files
pub fn compile(src_dir: &PathBuf, out_dir: &PathBuf) -> Result<(), Error> {
    let mut compiler = TSCompiler::new();
    // Walk through each file in source directory
    for entry in walkdir::WalkDir::new(src_dir) {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            let code = compiler.compile_file(entry.path());
            write_js_file(src_dir, out_dir, entry.path(), code.as_bytes());
        }
    }
    Ok(())
}
