// TODO: Make macros that define enumeration of exported names, so it
// can be imported by the host and will help to catch 'wrong export name' 
// kind of errors at compile time. The macro will define to_string method
// that will return the exact same name of an exported function which can 
// be passed to `get_function` fn of wasmer for example.

use crate::async_rt::AsyncRT;
use protocol::{Code, RunModuleFunctionParameters};
use runtime::runtime::JsRuntime;
use runtime::loader::ExerumLoader;
use runtime::cache::Memory;
use runtime::resolver::ExerumResolver;
use runtime::rquickjs::{Value, Function, Module};
use transpilers::{register, AssetTranspiler, TKey};
#[cfg(features = "ts")]
use transpiler_typescript::TypescriptTranspiler;
#[cfg(features = "jsx")]
use transpiler_jsx::JsxTranspiler;
use transpilers::Transpilers;
use transpiler_js::JsTranspiler;

/// 48 MB of preallocated memory to exchange parameters and return values.
pub(crate) const WASM_MEMORY_BUFFER_SIZE: usize = 48 * 1024 * 1024;
pub(crate) static mut WASM_MEMORY_BUFFER: [u8; WASM_MEMORY_BUFFER_SIZE] = [0; WASM_MEMORY_BUFFER_SIZE];

/// Returns a pointer to the buffer which is used to pass
/// function paramters and return values that take more space than 32 bits
#[export_name = "parameter_buffer_ptr"]
pub extern "C" fn parameter_buffer_ptr() -> u32 {
    let pointer: *const u8;
    unsafe {
        pointer = WASM_MEMORY_BUFFER.as_ptr();
    }
    return pointer as u32;
}

#[export_name = "new_async_runtime"]
pub extern "C" fn new_async_runtime() -> u32 {
    let async_rt = Box::new(AsyncRT::new());
    Box::into_raw(async_rt) as u32
}

#[export_name = "free_async_runtime"]
pub extern "C" fn free_async_runtime(async_rt_ptr: u32) {
    let async_rt = async_rt_ptr as *mut AsyncRT;
    unsafe { Box::from_raw(async_rt) };
}

/// There should be only one single javascript runtime per wasm module.
/// This function returns a pointer/handle that can be used to call other functions
/// to run javascript code.
#[export_name = "new_runtime"]
pub extern "C" fn new_runtime() -> u32 {
    let mut transpilers = Transpilers::default();
    #[cfg(feature = "ts")]
    register!(transpilers, "typescript", [.ts, .tsx], TypescriptTranspiler);
    #[cfg(feature = "jsx")]
    register!(transpilers, "javascript_react", [.jsx], JsxTranspiler);
    register!(transpilers, "javascript", [.js], JsTranspiler);
    let resolver = ExerumResolver::new(".");
    let loader = ExerumLoader::new(Box::new(Memory::default()), transpilers);
    let rt = Box::new(JsRuntime::new(loader, resolver));
    Box::into_raw(rt) as u32
}

/// Frees memory and all other resources taken by the javascript runtime.
#[export_name = "free_runtime"]
pub extern "C" fn free_runtime(rt_ptr: u32) {
    let rt = rt_ptr as *mut JsRuntime;
    unsafe { Box::from_raw(rt) };
}

/// Evaluates javascript code in a global context
/// 
/// # Arguments
/// `rt_ptr` - a pointer or handle returned by the `new_runtime`
/// `len` - the length of the utf-8 encoded javascript string in bytes.
///  
/// # Notes
/// The source code string must be copied to the address pointed by `parameter_buffer_ptr`
/// return value.
/// 
/// # Returns
/// Zero if no error occured.
#[export_name = "run"]
pub extern "C" fn run(async_rt_ptr: u32, jsrt_ptr: u32, len: usize) -> u32 {
    // Init function arguments
    let mut jsrt: Box<JsRuntime> = Box::from(jsrt_ptr);
    let async_rt: Box<AsyncRT> = Box::from(async_rt_ptr);
    let s = unsafe {
        String::from_utf8(WASM_MEMORY_BUFFER[0..len as usize].to_vec()).unwrap()
    };
    // Do work
    let context = jsrt.context();
    async_rt.block_on(async move {
        jsrt.spawn_executor();
        context.with(|ctx| {
            let _v: Value = ctx.eval(s).unwrap();
        });
        jsrt.rt().idle().await;
        // Don't drop JsRuntime
        Box::into_raw(jsrt);
    });
    // Don't drop AsyncRT
    Box::into_raw(async_rt);
    0
}

/// Compiles the javascript code to bytecode and writes it to the location pointed by `parameter_buffer_ptr`
/// return value.
/// 
/// # Arguments
/// `rt_ptr` - a pointer or handle returned by the `new_runtime`
/// `len` - the length of the utf-8 encoded javascript string in bytes.
/// 
/// # Returns
/// The size of the bytecode in bytes. Zero if error occured.
/// 
/// # Notes
/// The source code string must be copied to the address pointed by `parameter_buffer_ptr`
/// return value.
#[export_name = "compile_module"]
pub extern "C" fn compile_module(async_rt_ptr: u32, jsrt_ptr: u32, source_len: u32) -> u32 {
    // Init function arguments
    let mut jsrt: Box<JsRuntime> = Box::from(jsrt_ptr);
    let async_rt: Box<AsyncRT> = Box::from(async_rt_ptr);
    let module_source = unsafe {
        String::from_utf8(WASM_MEMORY_BUFFER[0..source_len as usize].to_vec()).unwrap()
    };
    // Do work
    let context = jsrt.context();
    let buffer = async_rt.block_on(async move {
        jsrt.spawn_executor();
        let res = context.with(|ctx| {
            let module = Module::new(ctx, "mod1", module_source.as_bytes()).unwrap();
            let buffer = module.write_object(false).unwrap();
            buffer
        });
        jsrt.rt().idle().await;
        Box::into_raw(jsrt);
        res
    });
    Box::into_raw(async_rt);
    // Write output value
    unsafe {
        WASM_MEMORY_BUFFER[0..buffer.len()].copy_from_slice(buffer.as_slice());
    };
    buffer.len() as u32
}

/// Evaluates javascript string as a named module for later use.
/// The parameter buffer should contain the name of the module followed by
/// source code encoded as utf-8 strings both.
/// # Arguments
/// `rt_ptr` - a pointer or handle returned by the `new_runtime`
/// `name_len` the length of the module name part in bytes.
/// `source_len` the length of the source code in bytes.
/// 
/// # Returns
/// Zero if no error occured
#[export_name = "eval_module"]
pub extern "C" fn eval_module(async_rt_ptr: u32, jsrt_ptr: u32, name_len: u32, source_len: u32) -> u32 {
    // Init function arguments
    let mut jsrt: Box<JsRuntime> = Box::from(jsrt_ptr);
    let async_rt: Box<AsyncRT> = Box::from(async_rt_ptr);
    let module_name = unsafe {
        String::from_utf8(WASM_MEMORY_BUFFER[0..name_len as usize].to_vec()).unwrap()
    };
    let module_source = unsafe {
        String::from_utf8(WASM_MEMORY_BUFFER[(name_len as usize)..(name_len as usize + source_len as usize)].to_vec()).unwrap()
    };
    // Do work
    let context = jsrt.context();
    async_rt.block_on(async move {
        jsrt.spawn_executor();
        context.with(|ctx| {
            let module = Module::new(ctx, module_name, module_source.as_bytes()).unwrap();
            module.eval().unwrap();
        });
        jsrt.rt().idle().await;
        Box::into_raw(jsrt);
    });
    Box::into_raw(async_rt);
    0
}

/// Evaluates javascript bytecode or string as a module and
/// runs a function from it.
/// # Arguments
/// Arguments are held in `protocol::RunModuleFunctionParamters`
/// # Notes
/// This function takes a bincode serialized structure wich might be 
/// too Rust specific.
/// May take bytecode or string as a source code
/// The return value of the function is written to the parameter buffer
/// as utf-8 encoded json string.
#[export_name = "run_module_function"]
pub extern "C" fn run_module_function(async_rt_ptr: u32, arguments_len: u32) -> u32 {
    // Init function arguments
    let mut async_rt: Box<AsyncRT> = Box::from(async_rt_ptr);
    let data = unsafe {
        &WASM_MEMORY_BUFFER[0..arguments_len as usize]
    };
    let parameters: RunModuleFunctionParameters = bincode::deserialize(data).unwrap();
    let mut jsrt: Box<JsRuntime> = Box::from(parameters.rt());
    // Do work
    let result = match execute_module_function(&mut async_rt, &mut jsrt, parameters) {
        Ok(r) => r,
        Err(_) => {
            return 0;
        }
    };
    Box::into_raw(jsrt);
    Box::into_raw(async_rt);
    // Write output value
    unsafe {
        WASM_MEMORY_BUFFER[0..result.as_bytes().len()].copy_from_slice(result.as_bytes());
    }
    result.as_bytes().len() as u32
}

pub fn execute_module_function(
    async_rt: &mut AsyncRT,
    rt: &mut JsRuntime,
    parameters: RunModuleFunctionParameters,
) -> Result<String, ()> {
    let context = rt.context();
    async_rt.block_on(async move {
        rt.spawn_executor();
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