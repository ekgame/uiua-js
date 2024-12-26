mod backend;

use std::time::Duration;

use backend::ExternalBackendHandlers;
use js_sys::Function;
use serde::de::Deserializer;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::Deserialize;
use uiua::encode::SmartOutput;
use uiua::format::{format_str, FormatConfig, FormatOutput};
use uiua::{
    Boxed, CodeSpan, Compiler, Diagnostic, InputSrc, Loc, SafeSys, Span, TraceFrame, Uiua, UiuaError, Value
};
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
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

#[derive(Clone)]
pub enum UiuaInputSource {
    String(usize),
    File(String),
    Macro(Box<DocumentSpan>),
    Builtin,
}

impl From<InputSrc> for UiuaInputSource {
    fn from(src: InputSrc) -> Self {
        match src {
            InputSrc::Str(id) => UiuaInputSource::String(id),
            InputSrc::File(path) => UiuaInputSource::File((*path).to_string_lossy().to_string()),
            InputSrc::Macro(src) => UiuaInputSource::Macro(Box::new(DocumentSpan::from(*src))),
        }
    }
}

impl Serialize for UiuaInputSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            UiuaInputSource::String(id) => {
                let mut struct_ser = serializer.serialize_struct("UiuaInputSource", 2)?;
                struct_ser.serialize_field("type", &"string")?;
                struct_ser.serialize_field("id", id)?;
                struct_ser.end()
            }
            UiuaInputSource::File(path) => {
                let mut struct_ser = serializer.serialize_struct("UiuaInputSource", 2)?;
                struct_ser.serialize_field("type", &"file")?;
                struct_ser.serialize_field("path", path)?;
                struct_ser.end()
            }
            UiuaInputSource::Macro(span) => {
                let mut struct_ser = serializer.serialize_struct("UiuaInputSource", 2)?;
                struct_ser.serialize_field("type", &"macro")?;
                struct_ser.serialize_field("span", span)?;
                struct_ser.end()
            }
            UiuaInputSource::Builtin => {
                let mut struct_ser = serializer.serialize_struct("UiuaInputSource", 1)?;
                struct_ser.serialize_field("type", &"builtin")?;
                struct_ser.end()
            }
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct DocumentSpan {
    pub src: UiuaInputSource,
    pub from: DocumentLocation,
    pub to: DocumentLocation,
}

impl DocumentSpan {
    fn fix_column(&self) -> Self {
        DocumentSpan {
            src: self.src.clone().into(),
            from: self.from.decrement_column(),
            to: self.to.decrement_column(),
        }
    }
}

impl From<CodeSpan> for DocumentSpan {
    fn from(span: CodeSpan) -> Self {
        DocumentSpan {
            src: span.src.into(),
            from: span.start.into(),
            to: span.end.into(),
        }
    }
}

impl From<Span> for DocumentSpan {
    fn from(span: Span) -> Self {
        match span {
            Span::Code(span) => DocumentSpan::from(span),
            Span::Builtin => DocumentSpan {
                src: UiuaInputSource::Builtin,
                from: DocumentLocation { line: 0, column: 0 },
                to: DocumentLocation { line: 0, column: 0 },
            },
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

#[derive(serde::Serialize)]
pub struct GlyphMapping {
    pub span_from: DocumentSpan,
    pub span_to: DocumentSpan,
}

impl From<(&CodeSpan, (Loc, Loc))> for GlyphMapping {
    fn from((span, (from, to)): (&CodeSpan, (Loc, Loc))) -> Self {
        GlyphMapping {
            span_from: DocumentSpan::from(span.clone()).fix_column(),
            span_to: DocumentSpan {
                src: span.src.clone().into(),
                from: DocumentLocation::from(from).add_line(1),
                to: DocumentLocation::from(to).add_line(1),
            },
        }
    }
}

#[derive(serde::Serialize)]
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
        let mut value = match &self {
            UiuaValue::Byte(array) => Value::Byte(uiua::Array::new(
                array.shape.clone().into_iter().collect::<uiua::Shape>(),
                array.data.as_slice(),
            )),
            UiuaValue::Num(array) => Value::Num(uiua::Array::new(
                array.shape.clone().into_iter().collect::<uiua::Shape>(),
                array.data.as_slice(),
            )),
            UiuaValue::Char(array) => Value::Char(uiua::Array::new(
                array.shape.clone().into_iter().collect::<uiua::Shape>(),
                array.data.as_slice(),
            )),
            UiuaValue::Complex(array) => Value::Complex(uiua::Array::new(
                array.shape.clone().into_iter().collect::<uiua::Shape>(),
                array
                    .data
                    .iter()
                    .map(|(re, im)| uiua::Complex { re: *re, im: *im })
                    .collect::<Vec<uiua::Complex>>()
                    .as_slice(),
            )),
            UiuaValue::Box(array) => Value::Box(uiua::Array::new(
                array.shape.clone().into_iter().collect::<uiua::Shape>(),
                array
                    .data
                    .iter()
                    .map(|v| Boxed((**v).clone().into()))
                    .collect::<Vec<Boxed>>()
                    .as_slice(),
            )),
        };

        let label = match &self {
            UiuaValue::Byte(array) => array.label.clone(),
            UiuaValue::Num(array) => array.label.clone(),
            UiuaValue::Char(array) => array.label.clone(),
            UiuaValue::Complex(array) => array.label.clone(),
            UiuaValue::Box(array) => array.label.clone(),
        };

        if let Some(label) = label {
            value.meta_mut().label = Some(label.into());
        }

        if let Some(keys) = match &self {
            UiuaValue::Byte(array) => array.keys.clone(),
            UiuaValue::Num(array) => array.keys.clone(),
            UiuaValue::Char(array) => array.keys.clone(),
            UiuaValue::Complex(array) => array.keys.clone(),
            UiuaValue::Box(array) => array.keys.clone(),
        } {
            value.map((*keys).into(), &Uiua::with_safe_sys()).unwrap();
        }

        value
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

#[derive(Deserialize, Debug)]
struct UiuaArrayStruct {
    data: Vec<serde_json::Value>,
    shape: Vec<usize>,
    label: Option<String>,
    keys: Option<Box<UiuaArrayStruct>>,
    #[serde(rename = "type")]
    type_: String,
}

impl From<UiuaArrayStruct> for UiuaValue {
    fn from(array: UiuaArrayStruct) -> Self {
        // web_sys::console::log_1(&format!("Converting array {:?}", array).into());
        let keys = array.keys.map(|keys| Box::new(UiuaValue::from(*keys)));
        match array.type_.as_str() {
            "number" => {
                let data = array
                    .data
                    .iter()
                    .map(|v| v.as_f64().unwrap())
                    .collect::<Vec<f64>>();
                UiuaValue::Num(UiuaArray {
                    data,
                    shape: array.shape,
                    label: array.label,
                    keys: keys,
                })
            }
            "char" => {
                let data = array
                    .data
                    .iter()
                    .map(|v| v.as_str().unwrap().chars().next().unwrap())
                    .collect::<Vec<char>>();
                UiuaValue::Char(UiuaArray {
                    data,
                    shape: array.shape,
                    label: array.label,
                    keys: keys,
                })
            }
            "complex" => {
                let data = array
                    .data
                    .iter()
                    .map(|v| {
                        let re = v[0].as_f64().unwrap();
                        let im = v[1].as_f64().unwrap();
                        (re, im)
                    })
                    .collect::<Vec<(f64, f64)>>();
                UiuaValue::Complex(UiuaArray {
                    data,
                    shape: array.shape,
                    label: array.label,
                    keys: keys,
                })
            }
            "box" => {
                let data = array
                    .data
                    .iter()
                    .map(|v| {
                        let value: UiuaValue = serde_json::from_value(v.clone()).unwrap();
                        Box::new(value)
                    })
                    .collect::<Vec<Box<UiuaValue>>>();
                UiuaValue::Box(UiuaArray {
                    data,
                    shape: array.shape,
                    label: array.label,
                    keys: keys,
                })
            }
            _ => panic!("Invalid type"),
        }
    }
}

impl<'de> Deserialize<'de> for UiuaValue {
    fn deserialize<D>(deserializer: D) -> Result<UiuaValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        let array: UiuaArrayStruct = Deserialize::deserialize(deserializer)?;
        Ok(array.into())
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
    execution_limit_seconds: Option<f64>,
}

#[wasm_bindgen]
impl UiuaRuntimeInternal {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        UiuaRuntimeInternal {
            bindings: Vec::new(),
            compiler: None,
            backend: ExternalBackendHandlers::default(),
            execution_limit_seconds: None,
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

    #[wasm_bindgen(js_name = setExecutionLimitSeconds)]
    pub fn set_execution_limit_seconds(&mut self, seconds: f64) {
        self.execution_limit_seconds = Some(seconds);
    }

    fn build_uiua_backend(&self) -> backend::CustomBackend {
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

    pub fn pop(&mut self) -> Result<JsValue, JsValue> {
        let uiua = unsafe { &mut *self.uiua };
        let result = uiua.pop(()).map(|value| UiuaValue::from(value));
        match result {
            Ok(value) => Ok(serde_wasm_bindgen::to_value(&value)?),
            Err(err) => Err(to_js_error(err)),
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
    diagnostics: Vec<Diagnostic>,
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

    #[wasm_bindgen(getter)]
    pub fn diagnostics(&self) -> JsValue {
        let diagnostics = self
            .diagnostics
            .iter()
            .map(|diagnostic| UiuaDiagnostic::from(diagnostic.clone()))
            .collect::<Vec<UiuaDiagnostic>>();

        serde_wasm_bindgen::to_value(&diagnostics).unwrap()
    }
}

#[derive(serde::Serialize)]
struct UiuaDiagnostic {
    pub message: String,
    pub span: DocumentSpan,
    pub kind: UiuaDiagnosticKind,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum UiuaDiagnosticKind {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "style")]
    Style,
    #[serde(rename = "advice")]
    Advice,
    #[serde(rename = "warning")]
    Warning,
}

impl From<Diagnostic> for UiuaDiagnostic {
    fn from(diagnostic: Diagnostic) -> Self {
        UiuaDiagnostic {
            message: diagnostic.message,
            span: DocumentSpan::from(diagnostic.span),
            kind: match diagnostic.kind {
                uiua::DiagnosticKind::Info => UiuaDiagnosticKind::Info,
                uiua::DiagnosticKind::Style => UiuaDiagnosticKind::Style,
                uiua::DiagnosticKind::Advice => UiuaDiagnosticKind::Advice,
                uiua::DiagnosticKind::Warning => UiuaDiagnosticKind::Warning,
            },
        }
    }
}

#[derive(serde::Serialize)]
struct UiuaTraceFrame {
    pub line: String,
    pub span: DocumentSpan,
}

impl From<TraceFrame> for UiuaTraceFrame {
    fn from(frame: TraceFrame) -> Self {
        let message = match (&frame.id, &frame.span) {
            (Some(id), Span::Code(span)) => format!("in {id} at {span}"),
            (Some(id), Span::Builtin) => format!("in {id}"),
            (None, Span::Code(span)) => format!("at {span}"),
            (None, Span::Builtin) => "<omitted>".to_owned(),
        };

        UiuaTraceFrame {
            line: message,
            span: DocumentSpan::from(frame.span),
        }
    }
}

#[derive(serde::Serialize)]
struct SimplifiedUiuaError {
    pub message: String,
    pub trace: Vec<UiuaTraceFrame>,
}

impl From<UiuaError> for SimplifiedUiuaError {
    fn from(err: UiuaError) -> Self {
        SimplifiedUiuaError {
            message: err.to_string(),
            trace: err
                .trace
                .iter()
                .map(|frame| UiuaTraceFrame::from(frame.clone()))
                .collect(),
        }
    }
}

fn to_js_error(err: UiuaError) -> JsValue {
    serde_wasm_bindgen::to_value(&SimplifiedUiuaError::from(err))
        .unwrap()
        .into()
}

#[wasm_bindgen(js_name = runCode)]
pub fn run_code(
    code: String,
    initial_values: Vec<JsValue>,
    runtime: UiuaRuntimeInternal,
) -> Result<UiuaExecutionResultInternal, JsValue> {
    let mut uiua = Uiua::with_safe_sys();

    if let Some(seconds) = runtime.execution_limit_seconds.clone() {
        uiua = uiua.with_execution_limit(Duration::from_secs_f64(seconds));
    }

    let backend = runtime.build_uiua_backend();

    let mut compiler: Compiler = match runtime.compiler.as_ref() {
        Some(compiler) => {
            let mut compiler = compiler.compiler.clone();
            compiler.set_backend(backend);
            compiler
        }
        None => Compiler::with_backend(backend),
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
        return Err(to_js_error(err));
    }

    initial_values.into_iter().for_each(|value| {
        let value: UiuaValue = serde_wasm_bindgen::from_value(value).unwrap();
        let value: Value = value.into();
        uiua.push(value);
    });

    let result = uiua.run_compiler(&mut compiler);

    if let Err(err) = result {
        return Err(to_js_error(err));
    }

    let diagnostics: Vec<Diagnostic> = compiler.take_diagnostics().into_iter().collect();
    let backend = uiua.downcast_backend::<backend::CustomBackend>().unwrap();
    let result = UiuaExecutionResultInternal {
        stack: uiua.stack().to_vec(),
        compiler,
        stdout: backend.stdout(),
        stderr: backend.stderr(),
        diagnostics,
    };

    Ok(result)
}

#[wasm_bindgen(js_name = prettyFormatValue)]
pub fn pretty_format_value(value: JsValue) -> Result<String, JsValue> {
    let value: UiuaValue = serde_wasm_bindgen::from_value(value)?;
    let value: Value = value.into();
    Ok(value.show())
}

enum UiuaSmartOutput {
    Png(Vec<u8>, Option<String>),
    Gif(Vec<u8>, Option<String>),
    Wav(Vec<u8>, Option<String>),
    Normal(UiuaValue)
}

impl Serialize for UiuaSmartOutput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            UiuaSmartOutput::Png(data, label) => {
                let mut struct_ser = serializer.serialize_struct("UiuaSmartOutput", 2)?;
                struct_ser.serialize_field("type", &"png")?;
                struct_ser.serialize_field("data", &data)?;
                struct_ser.serialize_field("label", &label)?;
                struct_ser.end()
            }
            UiuaSmartOutput::Gif(data, label) => {
                let mut struct_ser = serializer.serialize_struct("UiuaSmartOutput", 2)?;
                struct_ser.serialize_field("type", &"gif")?;
                struct_ser.serialize_field("data", &data)?;
                struct_ser.serialize_field("label", &label)?;
                struct_ser.end()
            }
            UiuaSmartOutput::Wav(data, label) => {
                let mut struct_ser = serializer.serialize_struct("UiuaSmartOutput", 2)?;
                struct_ser.serialize_field("type", &"wav")?;
                struct_ser.serialize_field("data", &data)?;
                struct_ser.serialize_field("label", &label)?;
                struct_ser.end()
            }
            UiuaSmartOutput::Normal(value) => {
                let mut struct_ser = serializer.serialize_struct("UiuaSmartOutput", 2)?;
                struct_ser.serialize_field("type", &"normal")?;
                struct_ser.serialize_field("data", &value)?;
                struct_ser.end()
            }
        }
    }
}

impl From<SmartOutput> for UiuaSmartOutput {
    fn from(output: SmartOutput) -> Self {
        match output {
            SmartOutput::Png(data, label) => UiuaSmartOutput::Png(data, label),
            SmartOutput::Gif(data, label) => UiuaSmartOutput::Gif(data, label),
            SmartOutput::Wav(data, label) => UiuaSmartOutput::Wav(data, label),
            SmartOutput::Normal(value) => UiuaSmartOutput::Normal(UiuaValue::from(value)),
            // TODO: Add SVG when that's fixed in Uiua
        }
    }
}

#[wasm_bindgen(js_name = toSmartValue)]
pub fn to_smart_value(value: JsValue) -> JsValue {
    let value: UiuaValue = serde_wasm_bindgen::from_value(value).unwrap();
    let smart_value = SmartOutput::from_value(value.into(), &SafeSys::default());
    serde_wasm_bindgen::to_value(&UiuaSmartOutput::from(smart_value)).unwrap()
}