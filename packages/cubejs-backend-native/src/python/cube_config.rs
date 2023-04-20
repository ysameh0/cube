use neon::prelude::*;

pub struct CubeConfigPy {
    pub(crate) schema_path: String,
    pub(crate) pg_sql_port: String,
}

impl Finalize for CubeConfigPy {}

impl CubeConfigPy {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_object<'a, C: Context<'a>>(self, cx: &mut C) -> JsResult<'a, JsObject> {
        let obj = cx.empty_object();

        let schema_path = JsString::new(cx, self.schema_path);
        obj.set(cx, "schema_path", schema_path)?;

        let pg_sql_port = JsString::new(cx, self.pg_sql_port);
        obj.set(cx, "pgSqlPort", pg_sql_port)?;

        Ok(obj)
    }
}
