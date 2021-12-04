use rquickjs::{Context, Runtime, Value, Tokio};
use rquickjs::{Loader, Resolver};
use stdlib::init_stdlib;

pub struct JsRuntime {
    rt: Runtime,
    executor_spawned: bool,
    pub(crate) context: Context,
}

impl JsRuntime {
    pub fn new(loader: impl Loader + 'static, resolver: impl Resolver + 'static) -> Self {
        let rt = Runtime::new().unwrap();
        rt.set_loader(resolver, loader);
        let context = Context::full(&rt).unwrap();
        init_stdlib(&context);
        JsRuntime { rt, context, executor_spawned: false }
    }

    pub fn rt(&self) -> &Runtime {
        &self.rt
    }

    #[inline]
    pub fn spawn_executor(&mut self) {
        if !self.executor_spawned {
            self.rt.spawn_executor(Tokio);
            self.executor_spawned = true;
        }
    }

    /// Adds another reference to the context
    pub fn context(&self) -> Context {
        self.context.clone()
    }

    /// Evaluates javascript at the global context
    pub fn run(&mut self, code: &str) {
        self.context.with(|ctx| {
            let _v: Value = ctx.eval(code).unwrap();
        });
    }
}

/// Returns default loader and resolver
pub fn _init_default_loader_and_resolver() -> (impl Loader, impl Resolver) {
    use rquickjs::{FileResolver, ScriptLoader};
    let resolver = FileResolver::default().with_path("./");
    let loader = ScriptLoader::default();
    (loader, resolver)
}

#[cfg(all(target_os = "wasi", target_arch = "wasm32"))]
impl From<u32> for Box<JsRuntime> {
    fn from(ptr: u32) -> Box<JsRuntime> {
        let jsrt = ptr as *mut JsRuntime;
        unsafe {
            Box::from_raw(jsrt)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::NoCache;
    use rquickjs::{Promise, Tokio};
    use transpilers::Transpilers;
    #[test]
    fn test_reusable_runtime() {
        let tokio_rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .enable_time()
            .build()
            .unwrap();
        let local_set = tokio::task::LocalSet::new();
        let transpilers = Transpilers::default();
        let resolver = crate::resolver::ExerumResolver::new("./test_data/");
        let loader = crate::loader::ExerumLoader::new(Box::new(NoCache {}), transpilers);
        let jsrt = Box::new(crate::runtime::JsRuntime::new(loader, resolver));
        let ctx = jsrt.context();

        // 1) Execute some simple js.
        let fut = local_set.run_until(async move {
            // Spawn executor before first usage.
            jsrt.rt().spawn_executor(Tokio);
            let _v: String = ctx.with(|ctx| {
                ctx.eval(r#""STRING""#).unwrap()
            });
            jsrt.rt().idle().await;
            // Don't drop the runtime.
            Box::into_raw(jsrt)
        });
        let rt = tokio_rt.block_on(fut);

        // 2) Reuse the JsRuntime.
        let jsrt: Box<crate::runtime::JsRuntime> = unsafe { Box::from_raw(rt) };
        let ctx = jsrt.context();
        let fut = local_set.run_until(async move {
            // Next time we don't have to spawn executor
            // jsrt.rt().spawn_executor(Tokio);
            let promise: Promise<String> = ctx.with(|ctx| {
                ctx.eval(r#"async function test() { return "S" }; test()"#).unwrap()
            });
            let res = promise.await.unwrap();
            assert_eq!(res, "S");
            jsrt.rt().idle().await;
        });
        tokio_rt.block_on(fut);
    }

    #[test]
    fn test_rt() {

        let tokio_rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .enable_time()
            .build()
            .unwrap();
        let local_set = tokio::task::LocalSet::new();

        // TODO: error when enabling "macro" feature for rquickjs
        // Updating crates.io index
        // error: failed to select a version for `syn`.
        //     ... required by package `darling_core v0.13.0`
        //     ... which satisfies dependency `darling_core = "=0.13.0"` of package `darling v0.13.0`
        //     ... which satisfies dependency `darling = "^0.13"` of package `rquickjs-macro v0.1.3 (https://github.com/exerum/quickrs?branch=wasm32-wasi#9341972b)`
        //     ... which satisfies git dependency `rquickjs-macro` of package `rquickjs v0.1.3 (https://github.com/exerum/quickrs?branch=wasm32-wasi#9341972b)`
        //     ... which satisfies git dependency `rquickjs` (locked to 0.1.3) of package `exerum-wasm32-wasi v0.1.0 (/home/kein/Work/js-runtime/exerum-wasm32-wasi)`
        // versions that meet the requirements `^1.0.69` are: 1.0.82, 1.0.81, 1.0.80, 1.0.79, 1.0.78, 1.0.77, 1.0.76, 1.0.75, 1.0.74, 1.0.73, 1.0.72, 1.0.71, 1.0.70, 1.0.69

        // all possible versions conflict with previously selected packages.

        //   previously selected package `syn v1.0.65`
        //     ... which satisfies dependency `syn = "^1"` (locked to 1.0.65) of package `ast_node v0.7.4`
        //     ... which satisfies dependency `ast_node = "^0.7.3"` (locked to 0.7.4) of package `swc_common v0.14.7`
        //     ... which satisfies dependency `swc_common = "^0.14.0"` (locked to 0.14.7) of package `swc v0.89.0`
        //     ... which satisfies dependency `swc = "^0.89"` (locked to 0.89.0) of package `swc-tools v0.1.0 (/home/kein/Work/js-runtime/swc-tools)`
        //     ... which satisfies path dependency `swc-tools` (locked to 0.1.0) of package `transpiler-jsx v0.1.0 (/home/kein/Work/js-runtime/transpiler-jsx)`
        //     ... which satisfies path dependency `transpiler-jsx` (locked to 0.1.0) of package `runtime v0.1.0 (/home/kein/Work/js-runtime/runtime)`
        // #[bind(object)]
        // pub async fn sleep(msecs: u64) {
        //     tokio::task::sleep(
        //         std::time::Duration::from_millis(msecs)
        //     ).await;
        // }
        let transpilers = Transpilers::default();
        let resolver = crate::resolver::ExerumResolver::new("./test_data/");
        let loader = crate::loader::ExerumLoader::new(Box::new(NoCache {}), transpilers);
        let jsrt = crate::runtime::JsRuntime::new(loader, resolver);
        let ctx = jsrt.context();
        let fut = local_set.run_until(async move {
            jsrt.rt().spawn_executor(Tokio);
            // ctx.with(|ctx| {
            // ctx.globals().init_def::<Sleep>().unwrap();
            // });

            let promise: Promise<String> = ctx.with(|ctx| {
                ctx.eval(
                    r#"
            async function test() {
                return "ok";
            }
            test()
        "#,
                )
                .unwrap()
            });

            let res = promise.await.unwrap();
            assert_eq!(res, "ok");

            jsrt.rt().idle().await;
        });
        tokio_rt.block_on(fut);
    }
}
