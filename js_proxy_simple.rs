
#[cfg(feature = "js")]
fn js_row_get(_this: &boa_engine::JsValue, args: &[boa_engine::JsValue], context: &mut boa_engine::Context) -> boa_engine::JsResult<boa_engine::JsValue> {
    let key = args.get(1).unwrap_or(&boa_engine::JsValue::undefined()).to_string(context)?;
    let col_name = key.to_std_string_escaped();
    let mut val = None;
    let mut found = false;

    crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
        if let Some(schema_ptr) = *s.borrow() {
            let schema = unsafe { &*schema_ptr };
            if let Some(idx) = schema.get_column_index(&col_name) {
                found = true;
                
                let mut used_new = false;
                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow() {
                        let row = unsafe { &*row_ptr };
                        if let Some(v) = row.get(idx) {
                            val = Some(convert_oxibase_to_boa(v, context));
                            used_new = true;
                        }
                    }
                });
                
                if !used_new {
                    crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
                        if let Some(row_ptr) = *r.borrow() {
                            let row = unsafe { &*row_ptr };
                            if let Some(v) = row.get(idx) {
                                val = Some(convert_oxibase_to_boa(v, context));
                            }
                        }
                    });
                }
            }
        }
    });

    if !found {
        return Ok(boa_engine::JsValue::undefined());
    }
    val.unwrap_or(Ok(boa_engine::JsValue::null()))
}

#[cfg(feature = "js")]
fn js_row_set(_this: &boa_engine::JsValue, args: &[boa_engine::JsValue], context: &mut boa_engine::Context) -> boa_engine::JsResult<boa_engine::JsValue> {
    let key = args.get(1).unwrap_or(&boa_engine::JsValue::undefined()).to_string(context)?;
    let undefined = boa_engine::JsValue::undefined();
    let val = args.get(2).unwrap_or(&undefined);
    let col_name = key.to_std_string_escaped();
    
    let mut found = false;
    let mut internal_err = None;

    crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
        if let Some(schema_ptr) = *s.borrow() {
            let schema = unsafe { &*schema_ptr };
            if let Some(idx) = schema.get_column_index(&col_name) {
                found = true;
                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow_mut() {
                        let row = unsafe { &mut *row_ptr };
                        if let Some(col) = schema.get_column(idx) {
                            match convert_boa_to_oxibase(val, &col.data_type, context) {
                                Ok(v) => { let _ = row.set(idx, v); },
                                Err(e) => internal_err = Some(e),
                            }
                        }
                    }
                });
            }
        }
    });

    if let Some(e) = internal_err {
        return Err(boa_engine::JsError::from_opaque(boa_engine::JsValue::new(boa_engine::JsString::from(e.to_string()))));
    }
    Ok(boa_engine::JsValue::new(true))
}

#[cfg(feature = "js")]
fn convert_oxibase_to_boa(value: &crate::core::Value, _context: &mut boa_engine::Context) -> boa_engine::JsResult<boa_engine::JsValue> {
    match value {
        crate::core::Value::Null(_) => Ok(boa_engine::JsValue::null()),
        crate::core::Value::Integer(i) => Ok(boa_engine::JsValue::new(*i as i32)), 
        crate::core::Value::Float(f) => Ok(boa_engine::JsValue::rational(*f)),
        crate::core::Value::Text(s) => Ok(boa_engine::JsValue::new(boa_engine::JsString::from(s.as_ref()))),
        crate::core::Value::Boolean(b) => Ok(boa_engine::JsValue::new(*b)),
        crate::core::Value::Timestamp(ts) => Ok(boa_engine::JsValue::new(boa_engine::JsString::from(ts.to_rfc3339()))),
        crate::core::Value::Json(j) => Ok(boa_engine::JsValue::new(boa_engine::JsString::from(j.as_ref()))),
    }
}

#[cfg(feature = "js")]
fn convert_boa_to_oxibase(value: &boa_engine::JsValue, dt: &crate::core::DataType, context: &mut boa_engine::Context) -> crate::core::Result<crate::core::Value> {
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
