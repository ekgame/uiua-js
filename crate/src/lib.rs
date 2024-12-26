mod backend;
mod formatting;
mod runtime;
mod value;

use value::NativeValueWrapper;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(js_name = prettyFormatValue)]
pub fn pretty_format_value(value: NativeValueWrapper) -> Result<String, JsValue> {
    Ok(value.to_value().show())
}
