use js_sys::Function;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use uiua::format::{format_str, FormatConfig, FormatOutput};
use uiua::{CodeSpan, Compiler, Loc, Uiua, Value};
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[wasm_bindgen]
pub struct FormatConfigStruct {
    trailing_newline: Option<bool>,
    comment_space_after_hash: Option<bool>,
    multiline_indent: Option<i32>,
    align_comments: Option<bool>,
    indent_item_imports: Option<bool>,
}

#[wasm_bindgen]
impl FormatConfigStruct {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_trailing_newline(mut self, trailing_newline: bool) -> Self {
        self.trailing_newline = Some(trailing_newline);
        self
    }

    pub fn with_comment_space_after_hash(mut self, comment_space_after_hash: bool) -> Self {
        self.comment_space_after_hash = Some(comment_space_after_hash);
        self
    }

    pub fn with_multiline_indent(mut self, multiline_indent: i32) -> Self {
        self.multiline_indent = Some(multiline_indent);
        self
    }

    pub fn with_align_comments(mut self, align_comments: bool) -> Self {
        self.align_comments = Some(align_comments);
        self
    }

    pub fn with_indent_item_imports(mut self, indent_item_imports: bool) -> Self {
        self.indent_item_imports = Some(indent_item_imports);
        self
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DocumentLocation {
    pub line: u16,
    pub column: u16,
}

impl DocumentLocation {
    pub fn add_line(&self, line: u16) -> Self {
        DocumentLocation {
            line: self.line + line,
            column: self.column,
        }
    }

    pub fn decrement_column(&self) -> Self {
        DocumentLocation {
            line: self.line,
            column: self.column - 1,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DocumentSpan {
    pub from: DocumentLocation,
    pub to: DocumentLocation,
}

impl DocumentSpan {
    fn fix_column(&self) -> Self {
        DocumentSpan {
            from: self.from.decrement_column(),
            to: self.to.decrement_column(),
        }
    }
}

impl From<CodeSpan> for DocumentSpan {
    fn from(span: CodeSpan) -> Self {
        DocumentSpan {
            from: span.start.into(),
            to: span.end.into(),
        }
    }
}

impl From<Loc> for DocumentLocation {
    fn from(loc: Loc) -> Self {
        DocumentLocation {
            line: loc.line,
            column: loc.col,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GlyphMapping {
    pub span_from: DocumentSpan,
    pub span_to: DocumentSpan,
}

impl From<(&CodeSpan, (Loc, Loc))> for GlyphMapping {
    fn from((span, (from, to)): (&CodeSpan, (Loc, Loc))) -> Self {
        GlyphMapping {
            span_from: DocumentSpan::from(span.clone()).fix_column(),
            span_to: DocumentSpan {
                from: DocumentLocation::from(from).add_line(1),
                to: DocumentLocation::from(to).add_line(1),
            },
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FormatOutputStruct {
    pub output: String,
    pub glyph_map: Vec<GlyphMapping>,
}

impl From<FormatOutput> for FormatOutputStruct {
    fn from(output: FormatOutput) -> Self {
        FormatOutputStruct {
            output: output.output,
            glyph_map: output
                .glyph_map
                .iter()
                .map(|(span, locs)| GlyphMapping::from((span, *locs)))
                .collect(),
        }
    }
}

#[wasm_bindgen]
pub fn format_internal(code: String, config: FormatConfigStruct) -> Result<JsValue, JsError> {
    let mut format_config = FormatConfig::default();

    if let Some(trailing_newline) = config.trailing_newline {
        format_config.trailing_newline = trailing_newline;
    }

    if let Some(comment_space_after_hash) = config.comment_space_after_hash {
        format_config.comment_space_after_hash = comment_space_after_hash;
    }

    if let Some(multiline_indent) = config.multiline_indent {
        format_config.multiline_indent = multiline_indent as usize;
    }

    if let Some(align_comments) = config.align_comments {
        format_config.align_comments = align_comments;
    }

    if let Some(indent_item_imports) = config.indent_item_imports {
        format_config.indent_item_imports = indent_item_imports;
    }

    let format_output = format_str(&*code, &format_config)?;
    let output = FormatOutputStruct::from(format_output);
    Ok(serde_wasm_bindgen::to_value(&output)?)
}

struct UiuaArray<T> {
    data: Vec<T>,
    shape: Vec<usize>,
    label: Option<String>,
    keys: Option<Box<UiuaValue>>,
}

enum UiuaValue {
    Byte(UiuaArray<u8>),
    Num(UiuaArray<f64>),
    Char(UiuaArray<char>),
    Complex(UiuaArray<(f64, f64)>),
    Box(UiuaArray<Box<UiuaValue>>),
}

impl Serialize for UiuaValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut struct_ser = serializer.serialize_struct("UiuaArray", 4)?;

        match self {
            UiuaValue::Byte(array) => {
                struct_ser.serialize_field("data", &array.data)?;
                struct_ser.serialize_field("shape", &array.shape)?;
                struct_ser.serialize_field("label", &array.label)?;
                struct_ser.serialize_field("keys", &array.keys)?;
                struct_ser.serialize_field("type", &"number")?;
            }
            UiuaValue::Num(array) => {
                struct_ser.serialize_field("data", &array.data)?;
                struct_ser.serialize_field("shape", &array.shape)?;
                struct_ser.serialize_field("label", &array.label)?;
                struct_ser.serialize_field("keys", &array.keys)?;
                struct_ser.serialize_field("type", &"number")?;
            }
            UiuaValue::Char(array) => {
                struct_ser.serialize_field("data", &array.data.iter().collect::<String>())?;
                struct_ser.serialize_field("shape", &array.shape)?;
                struct_ser.serialize_field("label", &array.label)?;
                struct_ser.serialize_field("keys", &array.keys)?;
                struct_ser.serialize_field("type", &"char")?;
            }
            UiuaValue::Complex(array) => {
                struct_ser.serialize_field("data", &array.data)?;
                struct_ser.serialize_field("shape", &array.shape)?;
                struct_ser.serialize_field("label", &array.label)?;
                struct_ser.serialize_field("keys", &array.keys)?;
                struct_ser.serialize_field("type", &"complex")?;
            }
            UiuaValue::Box(array) => {
                struct_ser.serialize_field("data", &array.data)?;
                struct_ser.serialize_field("shape", &array.shape)?;
                struct_ser.serialize_field("label", &array.label)?;
                struct_ser.serialize_field("keys", &array.keys)?;
                struct_ser.serialize_field("type", &"box")?;
            }
        }

        struct_ser.end()
    }
}

impl From<Value> for UiuaValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Byte(array) => UiuaValue::Byte(UiuaArray {
                data: array.elements().cloned().collect(),
                shape: array.shape().to_vec(),
                label: array.meta().label.as_ref().map(|s| s.to_string()),
                keys: array
                    .meta()
                    .map_keys
                    .as_ref()
                    .map(|keys| Box::new(keys.normalized_keys().into())),
            }),
            Value::Num(array) => UiuaValue::Num(UiuaArray {
                data: array.elements().cloned().collect(),
                shape: array.shape().to_vec(),
                label: array.meta().label.as_ref().map(|s| s.to_string()),
                keys: array
                    .meta()
                    .map_keys
                    .as_ref()
                    .map(|keys| Box::new(keys.normalized_keys().into())),
            }),
            Value::Char(array) => UiuaValue::Char(UiuaArray {
                data: array.elements().cloned().collect(),
                shape: array.shape().to_vec(),
                label: array.meta().label.as_ref().map(|s| s.to_string()),
                keys: array
                    .meta()
                    .map_keys
                    .as_ref()
                    .map(|keys| Box::new(keys.normalized_keys().into())),
            }),
            Value::Complex(array) => UiuaValue::Complex(UiuaArray {
                data: array.elements().map(|c| (c.re, c.im)).collect(),
                shape: array.shape().to_vec(),
                label: array.meta().label.as_ref().map(|s| s.to_string()),
                keys: array
                    .meta()
                    .map_keys
                    .as_ref()
                    .map(|keys| Box::new(keys.normalized_keys().into())),
            }),
            Value::Box(array) => UiuaValue::Box(UiuaArray {
                data: array
                    .elements()
                    .map(|v| Box::new(UiuaValue::from(v.0.clone())))
                    .collect(),
                shape: array.shape().to_vec(),
                label: array.meta().label.as_ref().map(|s| s.to_string()),
                keys: array
                    .meta()
                    .map_keys
                    .as_ref()
                    .map(|keys| Box::new(keys.normalized_keys().into())),
            }),
        }
    }
}

#[wasm_bindgen]
pub struct JsRuntime {
    bindings: Vec<JsBinding>,
}

#[wasm_bindgen]
impl JsRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        JsRuntime {
            bindings: Vec::new(),
        }
    }

    pub fn add_binding(&mut self, binding: JsBinding) {
        self.bindings.push(binding);
    }
}

#[wasm_bindgen]
pub struct JsBinding {
    name: String,
    signature: (usize, usize),
    callback: JsFunctionWrapper,
}

#[wasm_bindgen]
impl JsBinding {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, inouts: usize, outputs: usize, callback: Function) -> JsBinding {
        JsBinding {
            name,
            signature: (inouts, outputs),
            callback: JsFunctionWrapper(callback),
        }
    }
}

#[wasm_bindgen]
struct JsFunctionWrapper(Function);
unsafe impl Send for JsFunctionWrapper {}
unsafe impl Sync for JsFunctionWrapper {}

#[wasm_bindgen]
pub struct MyStruct {
    value: i32,
}

#[wasm_bindgen]
impl MyStruct {
    // Constructor
    #[wasm_bindgen(constructor)]
    pub fn new(value: i32) -> MyStruct {
        MyStruct { value }
    }

    // Getter
    pub fn get_value(&self) -> i32 {
        self.value
    }

    // Setter
    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }

    // Another method
    pub fn increment(&mut self) {
        self.value += 1;
    }
}

#[wasm_bindgen]
pub fn run(code: String, runtime: JsRuntime) -> Result<JsValue, JsValue> {
    let mut comp = Compiler::new();
    runtime.bindings.into_iter().for_each(|binding| {
        let callback = binding.callback;
        let _ = comp.create_bind_function(&binding.name, binding.signature, move |_| {
            let my_struct = MyStruct::new(42);
            callback
                .0
                .call1(&JsValue::undefined(), &JsValue::from(my_struct))
                .unwrap();
            Ok(())
        });
    });

    let result = comp.load_str(code.as_str());

    if let Err(err) = result {
        return Err(JsError::from(err).into());
    }

    let asm = comp.finish();
    let mut uiua = Uiua::with_safe_sys();
    let result = uiua.run_asm(asm);

    if let Err(err) = result {
        return Err(JsError::from(err).into());
    }

    let values = uiua
        .stack()
        .iter()
        .map(|value| UiuaValue::from(value.clone()))
        .collect::<Vec<UiuaValue>>();

    Ok(serde_wasm_bindgen::to_value(&values)?)
}
