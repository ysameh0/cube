use convert_case::{Case, Casing};
use neon::prelude::*;
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyBool, PyFloat, PyInt, PyString};
use pyo3::{PyAny, PyErr, PyResult};
use std::collections::HashMap;

pub enum CubeConfigPyVariableValue {
    String(String),
    Number(f64),
    Bool(bool),
}

pub struct CubeConfigPy {
    dynamic_properties: HashMap<String, CubeConfigPyVariableValue>,
}

impl CubeConfigPy {
    pub fn new() -> Self {
        Self {
            dynamic_properties: HashMap::new(),
        }
    }

    pub fn dynamic_from_attr(&mut self, config_module: &PyAny, key: &str) -> PyResult<()> {
        let v = config_module.getattr(&*key)?;
        if !v.is_none() {
            let value = if v.get_type().is_subclass_of::<PyString>()? {
                CubeConfigPyVariableValue::String(v.to_string())
            } else if v.get_type().is_subclass_of::<PyBool>()? {
                CubeConfigPyVariableValue::Bool(v.downcast::<PyBool>()?.is_true())
            } else if v.get_type().is_subclass_of::<PyFloat>()? {
                let f = v.downcast::<PyFloat>()?;
                CubeConfigPyVariableValue::Number(f.value())
            } else if v.get_type().is_subclass_of::<PyInt>()? {
                let i: i64 = v.downcast::<PyInt>()?.extract()?;
                CubeConfigPyVariableValue::Number(i as f64)
            } else {
                return Err(PyErr::new::<PyTypeError, _>(format!(
                    "Unsupported configuration type: {} for key: {}",
                    v.get_type(),
                    key
                )));
            };

            self.dynamic_properties
                .insert(key.to_case(Case::Camel), value);
        };

        Ok(())
    }
}

impl Finalize for CubeConfigPy {}

impl CubeConfigPy {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_object<'a, C: Context<'a>>(self, cx: &mut C) -> JsResult<'a, JsObject> {
        let obj = cx.empty_object();

        for (k, v) in self.dynamic_properties.into_iter() {
            match v {
                CubeConfigPyVariableValue::String(v) => {
                    let js_val = JsString::new(cx, v);
                    obj.set(cx, &*k, js_val)?;
                }
                CubeConfigPyVariableValue::Number(v) => {
                    let js_val = JsNumber::new(cx, v);
                    obj.set(cx, &*k, js_val)?;
                }
                CubeConfigPyVariableValue::Bool(v) => {
                    let js_val = JsBoolean::new(cx, v);
                    obj.set(cx, &*k, js_val)?;
                }
            }
        }

        Ok(obj)
    }
}
