use serde::{
    ser::{Serialize, SerializeStruct, Serializer},
    Deserialize, Deserializer,
};
use uiua::{encode::SmartOutput, Boxed, SafeSys, Uiua, Value};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[derive(Clone)]
pub struct UiuaArray<T> {
    data: Vec<T>,
    shape: Vec<usize>,
    label: Option<String>,
    keys: Option<Box<UiuaValue>>,
}

#[derive(Clone)]
pub enum UiuaValue {
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
pub struct UiuaArrayStruct {
    data: serde_json::Value,
    shape: Vec<usize>,
    label: Option<String>,
    keys: Option<Box<UiuaArrayStruct>>,
    #[serde(rename = "type")]
    type_: String,
}

impl From<UiuaArrayStruct> for UiuaValue {
    fn from(array: UiuaArrayStruct) -> Self {
        let keys = array.keys.map(|keys| Box::new(UiuaValue::from(*keys)));
        match array.type_.as_str() {
            "number" => {
                let data: Vec<f64> = serde_json::from_value(array.data).unwrap();
                UiuaValue::Num(UiuaArray {
                    data,
                    shape: array.shape,
                    label: array.label,
                    keys,
                })
            }
            "char" => {
                let data: String = serde_json::from_value(array.data).unwrap();
                UiuaValue::Char(UiuaArray {
                    data: data.chars().collect(),
                    shape: array.shape,
                    label: array.label,
                    keys,
                })
            }
            "complex" => {
                let data: Vec<(f64, f64)> = serde_json::from_value(array.data).unwrap();
                UiuaValue::Complex(UiuaArray {
                    data,
                    shape: array.shape,
                    label: array.label,
                    keys,
                })
            }
            "box" => {
                let data: Vec<UiuaValue> = serde_json::from_value(array.data).unwrap();
                UiuaValue::Box(UiuaArray {
                    data: data.into_iter().map(|v| Box::new(v)).collect(),
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

enum UiuaSmartOutput {
    Png(Vec<u8>, Option<String>),
    Gif(Vec<u8>, Option<String>),
    Wav(Vec<u8>, Option<String>),
    Normal(UiuaValue),
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
