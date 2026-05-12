fn js_row_get(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get(1).unwrap_or(&JsValue::undefined()).to_string(context)?;
    let col_name = key.to_std_string_escaped();
    
    let mut val = None;
    let mut found = false;

    crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
        if let Some(schema_ptr) = *s.borrow() {
            let schema = unsafe { &*schema_ptr };
            if let Some(idx) = schema.get_column_index(&col_name) {
                found = true;
                
                // For proxy target detection, we can pass a special marker property or just infer context from thread-locals
                // Since this is generic, we'll try NEW first, then OLD
                let mut used_new = false;
                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow() {
                        let row = unsafe { &*row_ptr };
                        if let Some(v) = row.get(idx) {
                });
            }
        }
    });

    if let Some(e) = internal_err {
        return Err(boa_engine::JsError::from_opaque(JsValue::new(JsString::from(e.to_string()))));
    }
    
    if !found {
        return Ok(JsValue::new(true)); // Best effort for unknown fields in JS
    }
    
    Ok(JsValue::new(true))
}

fn convert_oxibase_to_boa(value: &crate::core::Value, _context: &mut Context) -> JsResult<JsValue> {
    match value {
        crate::core::Value::Null(_) => Ok(JsValue::null()),
        crate::core::Value::Integer(i) => Ok(JsValue::new(*i as i32)), // Boa mostly uses i32/f64
        crate::core::Value::Float(f) => Ok(JsValue::rational(*f)),
        crate::core::Value::Text(s) => Ok(JsValue::new(JsString::from(s.as_ref()))),
        crate::core::Value::Boolean(b) => Ok(JsValue::new(*b)),
        crate::core::Value::Timestamp(ts) => Ok(JsValue::new(JsString::from(ts.to_rfc3339()))),
        crate::core::Value::Json(j) => Ok(JsValue::new(JsString::from(j.as_ref()))),
    }
}

fn convert_boa_to_oxibase(value: &JsValue, dt: &crate::core::DataType, context: &mut Context) -> Result<crate::core::Value, crate::core::Error> {
    if value.is_null_or_undefined() {
        return Ok(crate::core::Value::Null(dt.clone()));
    }

    match dt {
        crate::core::DataType::Integer => {
            if let Some(n) = value.as_number() {
                Ok(crate::core::Value::Integer(n as i64))
            } else {
                Err(crate::core::Error::internal("Cannot cast JS value to integer"))
            }
        },
        crate::core::DataType::Float => {
            if let Some(n) = value.as_number() {
                Ok(crate::core::Value::Float(n))
            } else {
                Err(crate::core::Error::internal("Cannot cast JS value to float"))
            }
        },
        crate::core::DataType::Boolean => {
            Ok(crate::core::Value::Boolean(value.to_boolean()))
        },
        _ => {
            if let Ok(s) = value.to_string(context) {
                Ok(crate::core::Value::text(s.to_std_string_escaped()))
            } else {
                Ok(crate::core::Value::text(value.display().to_string()))
            }
        }
    }
}
