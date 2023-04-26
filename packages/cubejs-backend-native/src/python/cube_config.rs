use crate::utils::bind_method;
use convert_case::{Case, Casing};
use cubesql::CubeError;
use neon::prelude::*;
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyBool, PyDict, PyFloat, PyFunction, PyInt, PyString};
use pyo3::{Py, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject};
use std::cell::RefCell;
use std::collections::HashMap;

pub enum CubeConfigPyVariableValue {
    String(String),
    Number(f64),
    Bool(bool),
}

pub struct CubeConfigPy {
    dynamic_properties: Option<HashMap<String, CubeConfigPyVariableValue>>,
    query_rewrite: Option<Py<PyFunction>>,
    check_auth: Option<Py<PyFunction>>,
}

type BoxedCubeConfigPy = JsBox<RefCell<CubeConfigPy>>;

impl CubeConfigPy {
    pub fn new() -> Self {
        Self {
            dynamic_properties: Some(HashMap::new()),
            query_rewrite: None,
            check_auth: None,
        }
    }

    pub fn apply_dynamic_functions(&mut self, config_module: &PyAny) -> PyResult<()> {
        self.query_rewrite = self.static_call_attr(config_module, "query_rewrite")?;
        self.check_auth = self.static_call_attr(config_module, "check_auth")?;

        Ok(())
    }

    pub fn static_call_attr<'a>(
        &mut self,
        config_module: &'a PyAny,
        key: &str,
    ) -> PyResult<Option<Py<PyFunction>>> {
        let v = config_module.getattr(&*key)?;
        if !v.is_none() {
            if v.get_type().is_subclass_of::<PyFunction>()? {
                let cb = v.downcast::<PyFunction>()?;
                println!("{:?}", v);

                let py: Py<PyFunction> = cb.into();
                return Ok(Some(py));
            } else {
                return Err(PyErr::new::<PyTypeError, _>(format!(
                    "Unsupported configuration type: {} for key: {}, must be a lambda",
                    v.get_type(),
                    key
                )));
            }
        }

        Ok(None)
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

            let mut dynamic_properties = self.dynamic_properties.take().unwrap();
            dynamic_properties.insert(key.to_case(Case::Camel), value);

            self.dynamic_properties = Some(dynamic_properties);
        };

        Ok(())
    }
}

impl Finalize for CubeConfigPy {}

fn convert_dict_from_js_to_python<'a, C: Context<'a>>(
    root: &PyDict,
    from: Handle<'a, JsObject>,
    cx: &mut C,
    py: Python,
) -> NeonResult<()> {
    let properties = from.get_own_property_names(cx)?;
    for i in 0..properties.len(cx) {
        let property: Handle<JsString> = properties.get(cx, i)?;

        // println!("prop {:?}", property.value(cx));

        root.set_item(property.value(cx), PyDict::new(py)).unwrap();
    }

    Ok(())
}

fn config_py_query_rewrite(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    #[cfg(build = "debug")]
    trace!("JsAsyncChannel.config_py_query_rewrite");

    let this = cx
        .this()
        .downcast_or_throw::<BoxedCubeConfigPy, _>(&mut cx)?;
    let query_arg = cx.argument::<JsObject>(0)?;
    let context_arg = cx.argument::<JsObject>(1)?;

    let (query, context) = Python::with_gil(|py| {
        let query = PyDict::new(py);
        convert_dict_from_js_to_python(query, query_arg, &mut cx, py).unwrap();

        let ctx = PyDict::new(py);
        convert_dict_from_js_to_python(ctx, context_arg, &mut cx, py).unwrap();

        (query.to_object(py), ctx.to_object(py))
    });

    let res = this.borrow_mut().query_rewrite(query, context);
    match res {
        Err(err) => cx.throw_error(format!("Python error: {}", err)),
        Ok(_) => Ok(cx.undefined()),
    }
}

fn config_py_check_auth(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    #[cfg(build = "debug")]
    trace!("JsAsyncChannel.config_py_check_auth");

    let this = cx
        .this()
        .downcast_or_throw::<BoxedCubeConfigPy, _>(&mut cx)?;
    let req_arg = cx.argument::<JsObject>(0)?;
    let authorization = cx.argument::<JsString>(1)?.value(&mut cx);

    let req = Python::with_gil(|py| {
        let req = PyDict::new(py);
        convert_dict_from_js_to_python(req, req_arg, &mut cx, py).unwrap();

        req.to_object(py)
    });

    let res = this.borrow_mut().check_auth(req, authorization);
    match res {
        Err(err) => cx.throw_error(format!("Python error: {}", err)),
        Ok(_) => Ok(cx.undefined()),
    }
}

impl CubeConfigPy {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_object<'a, C: Context<'a>>(mut self, cx: &mut C) -> JsResult<'a, JsObject> {
        let obj = cx.empty_object();

        let dynamic_properties = self.dynamic_properties.take().unwrap();
        for (k, v) in dynamic_properties.into_iter() {
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

        // before move
        let has_query_rewrite = self.query_rewrite.is_some();
        let has_check_auth = self.check_auth.is_some();
        // Pass JsAsyncChannel as this, because JsFunction cannot use closure (fn with move)
        let obj_this = cx.boxed(RefCell::new(self)).upcast::<JsValue>();

        if has_query_rewrite {
            let query_rewrite_fn = JsFunction::new(cx, config_py_query_rewrite)?;
            let query_rewrite = bind_method(cx, query_rewrite_fn, obj_this)?;
            obj.set(cx, "queryRewrite", query_rewrite)?;
        } else {
            // Validation fails with null on Cube side
            // let query_rewrite = JsNull::new(cx);
            // obj.set(cx, "queryRewrite", query_rewrite)?;
        };

        if has_check_auth {
            let check_auth_fn = JsFunction::new(cx, config_py_check_auth)?;
            let check_auth = bind_method(cx, check_auth_fn, obj_this)?;
            obj.set(cx, "checkAuth", check_auth)?;
        } else {
            // Validation fails with null on Cube side
            // let check_auth = JsNull::new(cx);
            // obj.set(cx, "checkAuth", check_auth)?;
        };

        Ok(obj)
    }

    fn query_rewrite(&mut self, query: PyObject, ctx: PyObject) -> Result<(), CubeError> {
        if let Some(query_rewrite_fn) = self.query_rewrite.as_ref() {
            Python::with_gil(|py| query_rewrite_fn.call1(py, (query, ctx))).unwrap();
        }

        Ok(())
    }

    fn check_auth(&mut self, req: PyObject, authorization: String) -> Result<(), CubeError> {
        if let Some(check_auth_fn) = self.check_auth.as_ref() {
            Python::with_gil(|py| check_auth_fn.call1(py, (req, &authorization))).unwrap();
        }

        Ok(())
    }
}
