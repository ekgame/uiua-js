use js_sys::{Object, Reflect};
use uiua::{encode::SmartOutput, SafeSys, Value};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
pub struct NativeValueWrapper(Value);

impl NativeValueWrapper {
    pub fn new(value: Value) -> Self {
        NativeValueWrapper(value)
    }

    pub fn to_value(&self) -> Value {
        self.0.clone()
    }
}

#[wasm_bindgen]
impl NativeValueWrapper {
    pub fn shape(&self) -> Vec<usize> {
        self.0.shape().to_vec()
    }

    pub fn label(&self) -> Option<String> {
        self.0.meta().label.as_ref().map(|s| s.to_string())
    }

    pub fn keys(&self) -> Option<NativeValueWrapper> {
        self.0
            .meta()
            .map_keys
            .as_ref()
            .map(|keys| NativeValueWrapper(keys.clone().normalized()))
    }

    pub fn data(&self) -> JsValue {
        match self.to_value() {
            Value::Byte(array) => JsValue::from(array.elements().cloned().collect::<Vec<u8>>()),
            Value::Num(array) => JsValue::from(array.elements().cloned().collect::<Vec<f64>>()),
            Value::Char(array) => JsValue::from(
                array
                    .elements()
                    .cloned()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>(),
            ),
            Value::Complex(array) => serde_wasm_bindgen::to_value(
                &array
                    .elements()
                    .map(|c| (c.re, c.im))
                    .collect::<Vec<(f64, f64)>>(),
            )
            .unwrap(),
            Value::Box(array) => JsValue::from(
                array
                    .elements()
                    .map(|v| JsValue::from(NativeValueWrapper(v.clone().into())))
                    .collect::<Vec<JsValue>>(),
            ),
        }
    }

    #[wasm_bindgen(js_name = type)]
    pub fn type_(&self) -> String {
        match self.to_value() {
            Value::Byte(_) | Value::Num(_) => "number".to_string(),
            Value::Char(_) => "char".to_string(),
            Value::Complex(_) => "complex".to_string(),
            Value::Box(_) => "box".to_string(),
        }
    }

    pub fn show(&self) -> String {
        self.0.show()
    }

    #[wasm_bindgen(js_name = smartValue)]
    pub fn smart_value(&self) -> JsValue {
        let output = SmartOutput::from_value(self.0.clone(), &SafeSys::default());
        let object = Object::new().into();

        match output {
            SmartOutput::Normal(value) => {
                Reflect::set(&object, &JsValue::from("type"), &JsValue::from("normal")).unwrap();
                Reflect::set(&object, &JsValue::from("value"), &JsValue::from(NativeValueWrapper(value))).unwrap();
            },
            SmartOutput::Png(vec, label) => {
                Reflect::set(&object, &JsValue::from("type"), &JsValue::from("png")).unwrap();
                Reflect::set(&object, &JsValue::from("value"), &JsValue::from(vec)).unwrap();
                Reflect::set(&object, &JsValue::from("label"), &JsValue::from(label)).unwrap();
            }
            SmartOutput::Gif(vec, label) => {
                Reflect::set(&object, &JsValue::from("type"), &JsValue::from("gif")).unwrap();
                Reflect::set(&object, &JsValue::from("value"), &JsValue::from(vec)).unwrap();
                Reflect::set(&object, &JsValue::from("label"), &JsValue::from(label)).unwrap();
            }
            SmartOutput::Wav(vec, label) => {
                Reflect::set(&object, &JsValue::from("type"), &JsValue::from("wav")).unwrap();
                Reflect::set(&object, &JsValue::from("value"), &JsValue::from(vec)).unwrap();
                Reflect::set(&object, &JsValue::from("label"), &JsValue::from(label)).unwrap();
            }
        };

        object
    }
}
