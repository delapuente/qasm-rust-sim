#[macro_use(lalrpop_mod)]
extern crate lalrpop_util;
extern crate cfg_if;
#[cfg(feature = "wasm")]
extern crate wasm_bindgen;
#[cfg(feature = "wasm")]
extern crate js_sys;
#[cfg(feature = "wasm")]
extern crate console_error_panic_hook;
extern crate num;
#[macro_use(approx_eq)]
extern crate float_cmp;
#[macro_use(cached, cached_key)]
extern crate cached;
extern crate rand;
lalrpop_mod!(pub open_qasm2, "/grammar/open_qasm2.rs");

mod grammar;
mod linker;
mod semantics;
pub mod complex;
pub mod statevector;
mod interpreter;
mod qe;

use std::collections::HashMap;
use std::iter::FromIterator;

#[cfg(feature = "wasm")]
use interpreter::computation::{ Computation, new_computation };

use cfg_if::cfg_if;

use linker::Linker;
use interpreter::runtime::ExecutionResult;

fn do_run(input: &str) -> Result<ExecutionResult, String> {
  let linker = Linker::with_embedded(HashMap::from_iter(vec![
    ("qelib1.inc".to_owned(), qe::QELIB1.to_owned())
  ]));
  let parser = open_qasm2::OpenQasmProgramParser::new();
  let program = parser.parse(&input).unwrap();
  let linked = linker.link(program).unwrap();
  interpreter::runtime::execute(&linked)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run(input: &str) -> Result<ExecutionResult, String> {
  do_run(input)
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

cfg_if! {
  if #[cfg(feature = "wee_alloc")] {
    extern crate wee_alloc;
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
  }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn run(input: &str) -> Computation {
  use statevector::wasm::as_float_array;
  use std::panic;
  panic::set_hook(Box::new(console_error_panic_hook::hook));
  let result = do_run(input).unwrap();
  new_computation(
    result.memory.iter().map(|(k, v)| (k.to_owned(), *v as f64)).collect(),
    as_float_array(&result.statevector)
  )
}
