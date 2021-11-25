// TODO: Make macros that define enumeration of exported names, so it
// can be imported by the host and will help to catch 'wrong export name' 
// kind of errors at compile time. The macro will define to_string method
// that will return the exact same name of an exported function which can 
// be passed to `get_function` fn of wasmer for example.

use protocol::{Code, RunModuleFunctionParameters};
use crate::runtimejs::runtime::JsRuntime;

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

/// There should be only one single javascript runtime per wasm module.
/// This function returns a pointer/handle that can be used to call other functions
/// to run javascript code.
#[export_name = "new_runtime"]
pub extern "C" fn new_runtime() -> u32 {
    let rt = Box::new(JsRuntime::new());
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
pub extern "C" fn run(rt_ptr: u32, len: usize) -> u32 {
    let rt = rt_ptr as *mut JsRuntime;
    let mut rt = unsafe { Box::from_raw(rt) };
    let s = unsafe {
        String::from_utf8(WASM_MEMORY_BUFFER[0..len as usize].to_vec()).unwrap()
    };
    rt.run(s);
    Box::into_raw(rt);
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
pub extern "C" fn compile_module(rt_ptr: u32, source_len: u32) -> u32 {
    let rt = rt_ptr as *mut JsRuntime;
    let rt = unsafe { Box::from_raw(rt) };
    let module_source = unsafe {
        String::from_utf8(WASM_MEMORY_BUFFER[0..source_len as usize].to_vec()).unwrap()
    };
    let buffer = rt.compile_module("mod1".to_owned(), module_source);
    unsafe {
        WASM_MEMORY_BUFFER[0..buffer.len()].copy_from_slice(buffer.as_slice());
    };
    Box::into_raw(rt);
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
pub extern "C" fn eval_module(rt_ptr: u32, name_len: u32, source_len: u32) -> u32 {
    let rt = rt_ptr as *mut JsRuntime;
    let rt = unsafe { Box::from_raw(rt) };
    let module_name = unsafe {
        String::from_utf8(WASM_MEMORY_BUFFER[0..name_len as usize].to_vec()).unwrap()
    };
    let module_source = unsafe {
        String::from_utf8(WASM_MEMORY_BUFFER[(name_len as usize)..(name_len as usize + source_len as usize)].to_vec()).unwrap()
    };
    rt.eval_module(module_name, module_source);
    Box::into_raw(rt);
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
pub extern "C" fn run_module_function(arguments_len: u32) -> u32 {
    let data = unsafe {
        &WASM_MEMORY_BUFFER[0..arguments_len as usize]
    };
    let parameters: RunModuleFunctionParameters = bincode::deserialize(data).unwrap();
    let rt = parameters.rt() as *mut JsRuntime;
    let rt = unsafe { Box::from_raw(rt) };
         let result = match rt.execute_module_function(parameters) {
                Ok(r) => r,
                Err(_) => {
                    return 0;
                }
            };
    Box::into_raw(rt);
    unsafe {
        WASM_MEMORY_BUFFER[0..result.as_bytes().len()].copy_from_slice(result.as_bytes());
    }
    result.as_bytes().len() as u32
}