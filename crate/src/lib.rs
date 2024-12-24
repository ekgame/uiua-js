mod backend;

use backend::ExternalBackendHandlers;
use js_sys::Function;
use serde::de::Deserializer;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::Deserialize;
use uiua::format::{format_str, FormatConfig, FormatOutput};
use uiua::{Boxed, CodeSpan, Compiler, IntoSysBackend, Loc, Uiua, Value};
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

#[derive(Clone)]
struct UiuaArray<T> {
    data: Vec<T>,
    shape: Vec<usize>,
    label: Option<String>,
    keys: Option<Box<UiuaValue>>,
}

#[derive(Clone)]
enum UiuaValue {
    Byte(UiuaArray<u8>),
    Num(UiuaArray<f64>),
    Char(UiuaArray<char>),
    Complex(UiuaArray<(f64, f64)>),
    Box(UiuaArray<Box<UiuaValue>>),
}

impl Into<Value> for UiuaValue {
    fn into(self) -> Value {
        match self {
            UiuaValue::Byte(array) => Value::Byte(uiua::Array::new(
                array.shape.into_iter().collect::<uiua::Shape>(),
                array.data.as_slice(),
            )),
            UiuaValue::Num(array) => Value::Num(uiua::Array::new(
                array.shape.into_iter().collect::<uiua::Shape>(),
                array.data.as_slice(),
            )),
            UiuaValue::Char(array) => Value::Char(uiua::Array::new(
                array.shape.into_iter().collect::<uiua::Shape>(),
                array.data.as_slice(),
            )),
            UiuaValue::Complex(array) => Value::Complex(uiua::Array::new(
                array.shape.into_iter().collect::<uiua::Shape>(),
                array
                    .data
                    .iter()
                    .map(|(re, im)| uiua::Complex { re: *re, im: *im })
                    .collect::<Vec<uiua::Complex>>()
                    .as_slice(),
            )),
            UiuaValue::Box(array) => Value::Box(uiua::Array::new(
                array.shape.into_iter().collect::<uiua::Shape>(),
                array
                    .data
                    .iter()
                    .map(|v| Boxed((**v).clone().into()))
                    .collect::<Vec<Boxed>>()
                    .as_slice(),
            )),
        }
    }
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

impl<'de> Deserialize<'de> for UiuaValue {
    fn deserialize<D>(deserializer: D) -> Result<UiuaValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct UiuaArrayStruct {
            data: Vec<serde_json::Value>,
            shape: Vec<usize>,
            label: Option<String>,
            keys: Option<Box<UiuaValue>>,
            #[serde(rename = "type")]
            type_: String,
        }

        let array: UiuaArrayStruct = Deserialize::deserialize(deserializer)?;

        match array.type_.as_str() {
            "number" => {
                let data = array
                    .data
                    .iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or(serde::de::Error::custom("Expected number"))
                    })
                    .collect::<Result<Vec<f64>, _>>()?;
                Ok(UiuaValue::Num(UiuaArray {
                    data,
                    shape: array.shape,
                    label: array.label,
                    keys: array.keys,
                }))
            }
            "char" => {
                let data = array
                    .data
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .and_then(|s| s.chars().next())
                            .ok_or(serde::de::Error::custom("Expected char"))
                    })
                    .collect::<Result<Vec<char>, _>>()?;
                Ok(UiuaValue::Char(UiuaArray {
                    data,
                    shape: array.shape,
                    label: array.label,
                    keys: array.keys,
                }))
            }
            "complex" => {
                let data = array
                    .data
                    .iter()
                    .map(|v| {
                        let re = v[0]
                            .as_f64()
                            .ok_or(serde::de::Error::custom("Expected number"))?;
                        let im = v[1]
                            .as_f64()
                            .ok_or(serde::de::Error::custom("Expected number"))?;
                        Ok((re, im))
                    })
                    .collect::<Result<Vec<(f64, f64)>, _>>()?;
                Ok(UiuaValue::Complex(UiuaArray {
                    data,
                    shape: array.shape,
                    label: array.label,
                    keys: array.keys,
                }))
            }
            "box" => {
                let data = array
                    .data
                    .iter()
                    .map(|v| {
                        let value: UiuaValue =
                            serde_json::from_value(v.clone()).map_err(serde::de::Error::custom)?;
                        Ok(Box::new(value))
                    })
                    .collect::<Result<Vec<Box<UiuaValue>>, _>>()?;
                Ok(UiuaValue::Box(UiuaArray {
                    data,
                    shape: array.shape,
                    label: array.label,
                    keys: array.keys,
                }))
            }
            _ => Err(serde::de::Error::custom("Invalid type")),
        }
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
                    .map(|keys| Box::new(keys.clone().normalized().into())),
            }),
            Value::Num(array) => UiuaValue::Num(UiuaArray {
                data: array.elements().cloned().collect(),
                shape: array.shape().to_vec(),
                label: array.meta().label.as_ref().map(|s| s.to_string()),
                keys: array
                    .meta()
                    .map_keys
                    .as_ref()
                    .map(|keys| Box::new(keys.clone().normalized().into())),
            }),
            Value::Char(array) => UiuaValue::Char(UiuaArray {
                data: array.elements().cloned().collect(),
                shape: array.shape().to_vec(),
                label: array.meta().label.as_ref().map(|s| s.to_string()),
                keys: array
                    .meta()
                    .map_keys
                    .as_ref()
                    .map(|keys| Box::new(keys.clone().normalized().into())),
            }),
            Value::Complex(array) => UiuaValue::Complex(UiuaArray {
                data: array.elements().map(|c| (c.re, c.im)).collect(),
                shape: array.shape().to_vec(),
                label: array.meta().label.as_ref().map(|s| s.to_string()),
                keys: array
                    .meta()
                    .map_keys
                    .as_ref()
                    .map(|keys| Box::new(keys.clone().normalized().into())),
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
                    .map(|keys| Box::new(keys.clone().normalized().into())),
            }),
        }
    }
}

#[wasm_bindgen]
pub struct UiuaRuntimeInternal {
    bindings: Vec<JsBinding>,
    compiler: Option<CompilerRef>,
    backend: ExternalBackendHandlers,
}

#[wasm_bindgen]
impl UiuaRuntimeInternal {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        UiuaRuntimeInternal {
            bindings: Vec::new(),
            compiler: None,
            backend: ExternalBackendHandlers::default(),
        }
    }

    #[wasm_bindgen(js_name = addBinding)]
    pub fn add_binding(&mut self, name: String, inputs: usize, outputs: usize, callback: Function) {
        let binding = JsBinding::new(name, inputs, outputs, callback);
        self.bindings.push(binding);
    }

    #[wasm_bindgen(js_name = setCompiler)]
    pub fn set_compiler(&mut self, compiler: &CompilerRef) {
        self.compiler = Some(compiler.clone());
    }

    #[wasm_bindgen(js_name = getBackend)]
    pub fn get_backend(&self) -> ExternalBackendHandlers {
        self.backend.clone()
    }

    #[wasm_bindgen(js_name = setBackend)]
    pub fn set_backend(&mut self, backend: ExternalBackendHandlers) {
        self.backend = backend;
    }

    fn build_uiua_backend(&self) -> impl IntoSysBackend {
        let mut backend = backend::CustomBackend::new();
        backend.set_backend(self.backend.clone());
        backend
    }
}

pub struct JsBinding {
    name: String,
    signature: (usize, usize),
    callback: JsFunctionWrapper,
}

impl JsBinding {
    pub fn new(name: String, inputs: usize, outputs: usize, callback: Function) -> JsBinding {
        JsBinding {
            name,
            signature: (inputs, outputs),
            callback: JsFunctionWrapper(callback),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
struct JsFunctionWrapper(Function);
unsafe impl Send for JsFunctionWrapper {}
unsafe impl Sync for JsFunctionWrapper {}

impl JsFunctionWrapper {
    fn call1(&self, this: &JsValue, arg: &JsValue) -> Result<JsValue, JsValue> {
        self.0.call1(this, arg)
    }
}

#[wasm_bindgen]
pub struct UiuaRef {
    uiua: *mut Uiua,
}

#[wasm_bindgen]
impl UiuaRef {
    fn new(uiua: &mut Uiua) -> UiuaRef {
        UiuaRef { uiua }
    }

    pub fn pop(&mut self) -> Result<JsValue, JsError> {
        let uiua = unsafe { &mut *self.uiua };
        let result = uiua.pop(()).map(|value| UiuaValue::from(value));
        match result {
            Ok(value) => Ok(serde_wasm_bindgen::to_value(&value)?),
            Err(err) => Err(JsError::from(err).into()),
        }
    }

    pub fn push(&mut self, value: JsValue) -> Result<(), JsError> {
        let uiua = unsafe { &mut *self.uiua };
        let value: UiuaValue = serde_wasm_bindgen::from_value(value)?;
        let value: Value = value.into();
        uiua.push(value);
        Ok(())
    }
}

#[wasm_bindgen]
pub struct UiuaExecutionResultInternal {
    stack: Vec<Value>,
    compiler: Compiler,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct CompilerRef {
    compiler: Compiler,
}

#[wasm_bindgen]
impl UiuaExecutionResultInternal {
    #[wasm_bindgen(getter)]
    pub fn stack(&self) -> JsValue {
        let values = self
            .stack
            .iter()
            .map(|value| UiuaValue::from(value.clone()))
            .collect::<Vec<UiuaValue>>();

        serde_wasm_bindgen::to_value(&values).unwrap()
    }

    #[wasm_bindgen(getter)]
    pub fn compiler(&self) -> CompilerRef {
        CompilerRef {
            compiler: self.compiler.clone(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn stdout(&self) -> Vec<u8> {
        self.stdout.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn stderr(&self) -> Vec<u8> {
        self.stderr.clone()
    }
}


#[wasm_bindgen(js_name = runCode)]
pub fn run_code(
    code: String,
    initial_values: Vec<JsValue>,
    runtime: UiuaRuntimeInternal,
) -> Result<UiuaExecutionResultInternal, JsValue> {
    let mut uiua = Uiua::with_safe_sys();
    
    let mut compiler: Compiler = match runtime.compiler.as_ref() {
        Some(compiler) => compiler.compiler.clone(),
        None => Compiler::with_backend(runtime.build_uiua_backend()),
    };

    runtime.bindings.into_iter().for_each(|binding| {
        let callback = binding.callback;
        let _ = compiler.create_bind_function(&binding.name, binding.signature, move |uiua| {
            let wrapped = UiuaRef::new(uiua);
            callback
                .call1(&JsValue::undefined(), &JsValue::from(wrapped))
                .unwrap();
            Ok(())
        });
    });

    // This line makes sure that if the compiler was used before, it won't rerun the previous code
    compiler.assembly_mut().root.clear();

    // Load the code into the compiler
    let result = compiler.load_str(code.as_str());

    if let Err(err) = result {
        return Err(JsError::from(err).into());
    }

    initial_values.into_iter().for_each(|value| {
        let value: UiuaValue = serde_wasm_bindgen::from_value(value).unwrap();
        let value: Value = value.into();
        uiua.push(value);
    });

    let result = uiua.run_compiler(&mut compiler);

    if let Err(err) = result {
        return Err(JsError::from(err).into());
    }

    let backend = uiua.downcast_backend::<backend::CustomBackend>().unwrap();
    let result = UiuaExecutionResultInternal {
        stack: uiua.stack().to_vec(),
        compiler,
        stdout: backend.stdout(),
        stderr: backend.stderr(),
    };

    Ok(result)
}
