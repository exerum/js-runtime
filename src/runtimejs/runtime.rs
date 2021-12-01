use crate::runtimejs::resolver::init_loader_and_resolver;
use crate::stdlib::init_stdlib;
use protocol::{Code, RunModuleFunctionParameters};
use rquickjs::{Context, Function, Module, Runtime, Value};
use std::future::Future;
use tokio::task;

/// Structure to work with javascript engine
/// Every function runs in tokio runtime
pub struct JsRuntime {
    rt: Runtime,
    tokio_rt: tokio::runtime::Runtime,
    pub(crate) context: Context,
}

impl JsRuntime {
    pub fn new() -> Self {
        let rt = Runtime::new().unwrap();
        let (loader, resolver) = init_loader_and_resolver();
        rt.set_loader(resolver, loader);
        let context = Context::full(&rt).unwrap();
        init_stdlib(&context);
        let tokio_rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        JsRuntime {
            rt,
            tokio_rt,
            context,
        }
    }

    fn run_in_tokio<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T> + 'static,
        T: 'static,
    {
        let local_set = task::LocalSet::new();
        let fut = local_set.run_until(async move {
            task::spawn_local(future).await.unwrap()
            // TODO: Process promises with ops here.
        });
        self.tokio_rt.block_on(fut)
    }

    /// Evaluates javascript at the global context
    pub fn run(&mut self, code: String) {
        let context = self.context.clone();
        self.run_in_tokio(async move {
            context.with(|ctx| {
                let _v: Value = ctx.eval(code).unwrap();
            });
        });
    }

    /// Compiles javascript code to bytecode
    pub fn compile_module(&self, name: String, code: String) -> Vec<u8> {
        let context = self.context.clone();
        self.run_in_tokio(async move {
            context.with(|ctx| {
                let module = Module::new(ctx, name, code.as_bytes()).unwrap();
                let buffer = module.write_object(false).unwrap();
                buffer
            })
        })
    }

    pub fn execute_module_function(
        &self,
        parameters: RunModuleFunctionParameters,
    ) -> Result<String, ()> {
        let context = self.context.clone();
        self.run_in_tokio(async move {
            context.with(|ctx| match parameters.code {
                Code::Bytecode(b) => {
                    let module = Module::read_object(ctx, b).unwrap();
                    let evaluated: Module = match module.eval() {
                        Ok(m) => m,
                        Err(err) => {
                            println!("Error evaluating bytecode: {:?}", err);
                            return Err(());
                        }
                    };
                    let f: Function = evaluated.get(&parameters.name).unwrap();
                    std::mem::forget(evaluated);
                    let result: Value = f.call((parameters.json,)).unwrap();
                    Ok(result.as_string().unwrap().to_string().unwrap())
                }
                Code::Text(s) => {
                    let evaluated: Module = match Module::new(ctx, parameters.name.as_bytes(), s) {
                        Ok(m) => m.eval().unwrap(),
                        Err(err) => {
                            println!("Error creating module from js: {:?}", err);
                            return Err(());
                        }
                    };
                    let f: Function = evaluated.get(&parameters.name).unwrap();
                    std::mem::forget(evaluated);
                    let result: Value = f.call((&parameters.json,)).unwrap();
                    Ok(result.as_string().unwrap().to_string().unwrap())
                }
            })
        })
    }

    /// Evaluates javascript code in a separate module context
    /// 
    /// # Errors
    /// panics on error
    pub fn eval_module(&self, name: String, code: String) {
        let context = self.context.clone();
        self.run_in_tokio(async move {
            context.with(|ctx| {
                let module = Module::new(ctx, name, code.as_bytes()).unwrap();
                module.eval().unwrap();
            })
        });
    }
}
