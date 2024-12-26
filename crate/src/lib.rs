mod backend;
mod formatting;
mod runtime;
mod value;

use uiua::Value;
use value::UiuaValue;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(js_name = prettyFormatValue)]
pub fn pretty_format_value(value: JsValue) -> Result<String, JsValue> {
    let value: UiuaValue = serde_wasm_bindgen::from_value(value)?;
    let value: Value = value.into();
    Ok(value.show())
}
