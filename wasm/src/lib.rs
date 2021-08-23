mod fmt;

use std::convert::TryFrom;
use std::error::Error as StdError;
use std::panic;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use intcode::run;

static COMPUTER: Lazy<Mutex<Option<run::Computer>>> = Lazy::new(Default::default);

#[derive(Debug, Serialize, Deserialize)]
pub enum State {
    Waiting,
    Complete,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    pub state: State,
    pub output: String,
}

fn to_js_value(e: impl StdError) -> JsValue {
    e.to_string().into()
}

#[wasm_bindgen(start)]
pub fn init() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    yansi::Paint::disable();
}

#[wasm_bindgen]
pub fn assemble(asm: &str) -> Result<JsValue, JsValue> {
    let opts = fmt::Html::new(&asm);
    let output = match intcode::assemble::to_intcode(&asm) {
        Ok((intcode, warnings)) => {
            let mut output = String::new();
            for warning in warnings {
                output.push_str(&opts.warning(&warning));
                output.push('\n');
            }
            *COMPUTER.lock().unwrap() = Some(run::Computer::new(intcode));
            Output {
                state: State::Waiting,
                output,
            }
        }
        Err((errors, warnings)) => {
            let mut output = String::new();
            for warning in warnings {
                output.push_str(&opts.warning(&warning));
                output.push('\n');
            }
            for error in errors {
                output.push_str(&opts.error(&error));
                output.push('\n');
            }
            Output {
                state: State::Complete,
                output,
            }
        }
    };
    JsValue::from_serde(&output).map_err(to_js_value)
}

#[wasm_bindgen]
pub fn next(input: Option<String>) -> Result<JsValue, JsValue> {
    let mut computer = COMPUTER.lock().unwrap();
    let computer = computer.as_mut().unwrap();
    let mut output = Vec::new();
    if let Some(i) = input {
        computer.feed(i.into_bytes().into_iter().map(i64::from));
    }
    let output = loop {
        match computer.next().map_err(to_js_value)? {
            run::State::Yielded(value) => {
                output.push(u8::try_from(value).map_err(to_js_value)?);
            }
            run::State::Waiting => {
                let output = String::from_utf8(output).map_err(to_js_value)?;
                break Output {
                    state: State::Waiting,
                    output,
                };
            }
            run::State::Complete => {
                let output = String::from_utf8(output).map_err(to_js_value)?;
                break Output {
                    state: State::Complete,
                    output,
                };
            }
        }
    };
    JsValue::from_serde(&output).map_err(to_js_value)
}
