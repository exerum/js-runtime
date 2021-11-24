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
#[export_name = "new_js_runtime"]
pub extern "C" fn new_js_runtime() -> u32 {
    0
}

/// Frees memory and all other resources taken by the javascript runtime.
#[export_name = "free_js_runtime"]
pub extern "C" fn free_js_runtime(_rt_ptr: u32) {}

/// Evaluates javascript code in a global context
/// 
/// # Arguments
/// `rt_ptr` - a pointer or handle returned by the `new_js_runtime`
/// `len` - the length of the utf-8 encoded javascript string in bytes.
///  
/// # Notes
/// The source code string must be copied to the address pointed by `parameter_buffer_ptr`
/// return value.
/// 
/// # Returns
/// Zero if no error occured.
#[export_name = "run_javascript"]
pub extern "C" fn run_javascript(_rt_ptr: *const u8, _len: usize) -> u32 {
    1
}

/// Compiles the javascript code to bytecode and writes it to the location pointed by `parameter_buffer_ptr`
/// return value.
/// 
/// # Arguments
/// `rt_ptr` - a pointer or handle returned by the `new_js_runtime`
/// `len` - the length of the utf-8 encoded javascript string in bytes.
/// 
/// # Returns
/// The size of the bytecode in bytes. Zero if error occured.
/// 
/// # Notes
/// The source code string must be copied to the address pointed by `parameter_buffer_ptr`
/// return value.
#[export_name = "compile_js_module"]
pub extern "C" fn compile_js_module(_rt_ptr: u32, _source_len: u32) -> u32 {
    0
}

/// Evaluates javascript string as a named module for later use.
/// The parameter buffer should contain the name of the module followed by
/// source code encoded as utf-8 strings both.
/// # Arguments
/// `rt_ptr` - a pointer or handle returned by the `new_js_runtime`
/// `name_len` the length of the module name part in bytes.
/// `source_len` the length of the source code in bytes.
/// 
/// # Returns
/// Zero if no error occured
#[export_name = "node_eval_module"]
pub extern "C" fn eval_js_module(_rt_ptr: u32, _name_len: u32, _source_len: u32) -> u32 {
    1
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
#[export_name = "run_js_module_function"]
pub extern "C" fn run_js_module_function(_arguments_len: u32) -> u32 {
    1
}