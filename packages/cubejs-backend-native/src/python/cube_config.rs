use crate::utils::bind_method;

use convert_case::{Case, Casing};
use cubesql::CubeError;
use log::error;
use neon::prelude::*;
use neon::types::JsDate;
use pyo3::exceptions::{PyNotImplementedError, PyTypeError};

use pyo3::types::{PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString};
use pyo3::{AsPyPointer, Py, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject};

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

    pub fn get_query_rewrite(&self) -> Result<&Py<PyFunction>, CubeError> {
        if let Some(fun) = self.query_rewrite.as_ref() {
            Ok(fun)
        } else {
            Err(CubeError::internal(
                "Unable to reference query_rewrite, it's empty".to_string(),
            ))
        }
    }

    pub fn get_check_auth(&self) -> Result<&Py<PyFunction>, CubeError> {
        if let Some(fun) = self.check_auth.as_ref() {
            Ok(fun)
        } else {
            Err(CubeError::internal(
                "Unable to reference check_auth, it's empty".to_string(),
            ))
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

fn convert_from_js_to_python<'a, C: Context<'a>>(
    from: Handle<'a, JsValue>,
    cx: &mut C,
    py: Python,
) -> NeonResult<PyObject> {
    if from.is_a::<JsString, _>(cx) {
        let v = from.downcast_or_throw::<JsString, _>(cx)?;
        Ok(PyString::new(py, &v.value(cx)).to_object(py))
    } else if from.is_a::<JsArray, _>(cx) {
        let v = from.downcast_or_throw::<JsArray, _>(cx)?;
        let el = v.to_vec(cx)?;

        let mut elements = Vec::with_capacity(el.len());

        for el in el {
            let py_el = convert_from_js_to_python(el, cx, py)?;
            elements.push(py_el)
        }

        Ok(PyList::new(py, elements).to_object(py))
    } else if from.is_a::<JsObject, _>(cx) {
        let r = PyDict::new(py);

        let v = from.downcast_or_throw::<JsObject, _>(cx)?;
        let properties = v.get_own_property_names(cx)?;
        for i in 0..properties.len(cx) {
            let property: Handle<JsString> = properties.get(cx, i)?;
            let property_val = v.get_value(cx, property)?;
            let to_val = convert_from_js_to_python(property_val, cx, py)?;

            r.set_item(property.value(cx), to_val).unwrap();
        }

        Ok(r.to_object(py))
    } else if from.is_a::<JsBoolean, _>(cx) {
        // TODO: Implement
        Ok(PyDict::new(py).to_object(py))
    } else if from.is_a::<JsNumber, _>(cx) {
        let v = from.downcast_or_throw::<JsNumber, _>(cx)?;
        Ok(PyFloat::new(py, v.value(cx)).to_object(py))
    } else if from.is_a::<JsNull, _>(cx) {
        Ok(py.None())
    } else if from.is_a::<JsUndefined, _>(cx) {
        Ok(py.None())
    } else if from.is_a::<JsPromise, _>(cx) {
        cx.throw_error(format!("Unsupported conversion from JsPromise to Py"))
    } else if from.is_a::<JsFunction, _>(cx) {
        cx.throw_error(format!("Unsupported conversion from JsFunction to Py"))
    } else if from.is_a::<JsDate, _>(cx) {
        cx.throw_error(format!("Unsupported conversion from JsDate to Py"))
    } else {
        cx.throw_error(format!("Unsupported conversion from {:?} to Py", from))
    }
}

fn config_py_query_rewrite(mut cx: FunctionContext) -> JsResult<JsPromise> {
    #[cfg(build = "debug")]
    trace!("JsAsyncChannel.config_py_query_rewrite");

    let (deferred, promise) = cx.promise();
    let channel = cx.channel();

    let this = cx
        .this()
        .downcast_or_throw::<BoxedCubeConfigPy, _>(&mut cx)?;
    let query_arg = cx.argument::<JsObject>(0)?;
    let context_arg = cx.argument::<JsObject>(1)?;

    let (query, context) = Python::with_gil(|py| {
        let query = convert_from_js_to_python(query_arg.as_value(&mut cx), &mut cx, py)?;
        let ctx = convert_from_js_to_python(context_arg.as_value(&mut cx), &mut cx, py)?;

        Ok((query, ctx))
    })?;

    let py_method = match this.borrow().get_query_rewrite() {
        Ok(fun) => fun.clone(),
        Err(err) => return cx.throw_error(format!("{}", err)),
    };
    std::thread::spawn(move || {
        let res = Python::with_gil(|py| {
            let res = py_method.call1(py, (query, context))?;
            let is_coroutine = unsafe { pyo3::ffi::PyCoro_CheckExact(res.as_ptr()) == 1 };
            if is_coroutine {
                // let fut = pyo3_asyncio::tokio::into_future(call_res.as_ref(py))?;
                // pyo3_asyncio::tokio::run(py, fut)

                Err(PyErr::new::<PyNotImplementedError, _>(
                    "Async functions are not supported, unimplemented",
                ))
            } else {
                Ok(res)
            }
        });
        deferred.settle_with(&channel, move |mut cx| match res {
            Err(err) => {
                error!("Python error: {:?}", err);

                cx.throw_error(format!("Python error: {}", err))
            }
            Ok(_) => Ok(cx.undefined()),
        });
    });

    Ok(promise)
}

fn config_py_check_auth(mut cx: FunctionContext) -> JsResult<JsPromise> {
    #[cfg(build = "debug")]
    trace!("JsAsyncChannel.config_py_check_auth");

    let (deferred, promise) = cx.promise();
    let channel = cx.channel();

    let this = cx
        .this()
        .downcast_or_throw::<BoxedCubeConfigPy, _>(&mut cx)?;

    let req_arg = cx.argument::<JsObject>(0)?;
    let authorization = cx.argument::<JsString>(1)?.value(&mut cx);

    let req = Python::with_gil(|py| {
        let req = convert_from_js_to_python(req_arg.as_value(&mut cx), &mut cx, py)?;

        Ok(req)
    })?;

    let py_method = match this.borrow().get_check_auth() {
        Ok(fun) => fun.clone(),
        Err(err) => return cx.throw_error(format!("{}", err)),
    };
    std::thread::spawn(move || {
        let res = Python::with_gil(|py| {
            let res = py_method.call1(py, (req, &authorization))?;
            let is_coroutine = unsafe { pyo3::ffi::PyCoro_CheckExact(res.as_ptr()) == 1 };
            if is_coroutine {
                // let fut = pyo3_asyncio::tokio::into_future(call_res.as_ref(py))?;
                // pyo3_asyncio::tokio::run(py, fut)

                Err(PyErr::new::<PyNotImplementedError, _>(
                    "Async functions are not supported, unimplemented",
                ))
            } else {
                Ok(res)
            }
        });
        deferred.settle_with(&channel, move |mut cx| match res {
            Err(err) => {
                error!("Python error: {:?}", err);

                cx.throw_error(format!("Python error: {}", err))
            }
            Ok(_) => Ok(cx.undefined()),
        });
    });

    Ok(promise)
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
}
