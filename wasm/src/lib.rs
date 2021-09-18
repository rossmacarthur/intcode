mod fmt;

use std::convert::TryFrom;
use std::error::Error as StdError;
use std::panic;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use intcode::{run, ErrorSet, Intcode};

static COMPUTER: Lazy<Mutex<Option<run::Computer>>> = Lazy::new(Default::default);

#[derive(Debug, Serialize)]
pub enum AssembleState {
    Running,
    Failed,
}

#[derive(Debug, Serialize)]
pub struct AssembleOutput {
    pub state: AssembleState,
    pub output: String,
    pub intcode: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum NextState {
    Waiting,
    Complete,
}

#[derive(Debug, Serialize)]
pub struct NextOutput {
    pub state: NextState,
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
    let opts = fmt::Html::new(asm);
    let output = match intcode::assemble::to_intcode(asm) {
        Ok(Intcode {
            output: intcode,
            warnings,
        }) => {
            let mut output = String::new();
            for warning in warnings {
                output.push_str(&opts.warning(&warning));
                output.push('\n');
            }
            let human_intcode = intcode
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(",");
            *COMPUTER.lock().unwrap() = Some(run::Computer::new(intcode));
            AssembleOutput {
                state: AssembleState::Running,
                output,
                intcode: Some(human_intcode),
            }
        }
        Err(ErrorSet { errors, warnings }) => {
            let mut output = String::new();
            for warning in warnings {
                output.push_str(&opts.warning(&warning));
                output.push('\n');
            }
            for error in errors {
                output.push_str(&opts.error(&error));
                output.push('\n');
            }
            AssembleOutput {
                state: AssembleState::Failed,
                output,
                intcode: None,
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
                break NextOutput {
                    state: NextState::Waiting,
                    output,
                };
            }
            run::State::Complete => {
                let output = String::from_utf8(output).map_err(to_js_value)?;
                break NextOutput {
                    state: NextState::Complete,
                    output,
                };
            }
        }
    };
    JsValue::from_serde(&output).map_err(to_js_value)
}
