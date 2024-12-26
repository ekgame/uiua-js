use std::sync::{Arc, Mutex};

use js_sys::{Function, JsString, Reflect};
use uiua::SysBackend;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::runtime::JsFunctionWrapper;

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

    pub fn stdout(&self) -> Vec<u8> {
        self.stdout.lock().unwrap().clone()
    }

    pub fn stderr(&self) -> Vec<u8> {
        self.stderr.lock().unwrap().clone()
    }
}

fn format_error(error: JsValue) -> String {
    if let Ok(message) = Reflect::get(&error, &JsString::from("message")) {
        return message.as_string().unwrap_or("Unknown error".to_string());
    }

    "Unknown error".to_string()
}

impl SysBackend for CustomBackend {
    fn any(&self) -> &dyn std::any::Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn print_str_stdout(&self, s: &str) -> Result<(), String> {
        self.stdout.lock().unwrap().extend_from_slice(s.as_bytes());

        if let Some(handler) = &self.backend.print_str_stdout_handler {
            if let Err(error) = handler.call1(&JsValue::undefined(), &JsValue::from(s)) {
                return Err(format_error(error));
            }
        }

        Ok(())
    }

    fn print_str_stderr(&self, s: &str) -> Result<(), String> {
        self.stderr.lock().unwrap().extend_from_slice(s.as_bytes());

        if let Some(handler) = &self.backend.print_str_stderr_handler {
            if let Err(error) = handler.call1(&JsValue::undefined(), &JsValue::from(s)) {
                return Err(format_error(error));
            }
        }

        Ok(())
    }
}
