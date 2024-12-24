use std::sync::{Arc, Mutex};

use js_sys::Function;
use uiua::SysBackend;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::JsFunctionWrapper;

#[wasm_bindgen]
#[derive(Default, Clone, Debug)]
pub struct ExternalBackendHandlers {
    print_str_stdout_handler: Option<JsFunctionWrapper>,
    print_str_stderr_handler: Option<JsFunctionWrapper>,
}

#[wasm_bindgen]
impl ExternalBackendHandlers {
    pub fn with_print_str_stdout_handler(mut self, handler: Function) -> Self {
        self.print_str_stdout_handler = Some(JsFunctionWrapper(handler));
        self
    }

    pub fn with_print_str_stderr_handler(mut self, handler: Function) -> Self {
        self.print_str_stderr_handler = Some(JsFunctionWrapper(handler));
        self
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct CustomBackend {
    stdout: Arc<Mutex<Vec<u8>>>,
    stderr: Arc<Mutex<Vec<u8>>>,
    backend: ExternalBackendHandlers,
}

impl CustomBackend {
    pub fn new() -> Self {
        CustomBackend::default()
    }

    pub fn set_backend(&mut self, backend: ExternalBackendHandlers) {
        self.backend = backend;
    }
}

impl SysBackend for CustomBackend {
    fn any(&self) -> &dyn std::any::Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn print_str_stdout(&self, s: &str) -> Result<(), String> {
        if let Ok(mut stdout) = self.stdout.lock() {
            stdout.extend_from_slice(s.as_bytes());
        } else {
            return Err("Failed to lock stdout".to_string());
        }

        if let Some(handler) = &self.backend.print_str_stdout_handler {
            let _ = handler.call1(&JsValue::undefined(), &JsValue::from(s));
        }

        Ok(())
    }

    fn print_str_stderr(&self, s: &str) -> Result<(), String> {
        if let Ok(mut stderr) = self.stderr.lock() {
            stderr.extend_from_slice(s.as_bytes());
        } else {
            return Err("Failed to lock stderr".to_string());
        }

        if let Some(handler) = &self.backend.print_str_stderr_handler {
            let _ = handler.call1(&JsValue::undefined(), &JsValue::from(s));
        }

        Ok(())
    }
}