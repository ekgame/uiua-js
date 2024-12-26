use std::time::Duration;

use js_sys::Function;
use uiua::{Compiler, Diagnostic, Span, TraceFrame, Uiua, UiuaError, Value};
use wasm_bindgen::{prelude::wasm_bindgen, JsError, JsValue};

use crate::{
    backend::{self, CustomBackend, ExternalBackendHandlers},
    formatting::DocumentSpan, value::NativeValueWrapper,
};

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

    fn build_uiua_backend(&self) -> CustomBackend {
        let mut backend = CustomBackend::new();
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

#[derive(Clone, Debug)]
pub struct JsFunctionWrapper(pub Function);
unsafe impl Send for JsFunctionWrapper {}
unsafe impl Sync for JsFunctionWrapper {}

impl JsFunctionWrapper {
    pub fn call1(&self, this: &JsValue, arg: &JsValue) -> Result<JsValue, JsValue> {
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

    pub fn pop(&mut self) -> Result<NativeValueWrapper, JsValue> {
        let uiua = unsafe { &mut *self.uiua };
        let result = uiua.pop(());
        match result {
            Ok(value) => Ok(NativeValueWrapper::new(value)),
            Err(err) => Err(to_js_error(err)),
        }
    }

    pub fn push(&mut self, value: NativeValueWrapper) -> Result<(), JsError> {
        let uiua = unsafe { &mut *self.uiua };
        uiua.push(value.to_value());
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
            .map(|value| NativeValueWrapper::new(value.clone()))
            .collect::<Vec<NativeValueWrapper>>();

        JsValue::from(values)
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

pub fn to_js_error(err: UiuaError) -> JsValue {
    serde_wasm_bindgen::to_value(&SimplifiedUiuaError::from(err))
        .unwrap()
        .into()
}

#[wasm_bindgen(js_name = runCode)]
pub fn run_code(
    code: String,
    initial_values: Vec<NativeValueWrapper>,
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
        uiua.push(value.to_value());
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
