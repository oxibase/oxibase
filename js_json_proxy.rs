    fn build_new_row_json(&self, context: &mut boa_engine::Context) -> Option<boa_engine::JsValue> {
        let mut js_val = None;
        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow() {
                        let row = unsafe { &*row_ptr };
                        let obj = boa_engine::object::JsObject::with_null_proto();
                        for col in &schema.columns {
                            if let Some(val) = row.get(col.id) {
                                if let Ok(jv) = self.convert_oxibase_to_boa(val) {
                                    let _ = obj.set(boa_engine::JsString::from(col.name.as_str()), jv, false, context);
                                }
                            }
                        }
                        js_val = Some(boa_engine::JsValue::from(obj));
                    }
                });
            }
        });
        js_val
    }

    fn build_old_row_json(&self, context: &mut boa_engine::Context) -> Option<boa_engine::JsValue> {
        let mut js_val = None;
        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow() {
                        let row = unsafe { &*row_ptr };
                        let obj = boa_engine::object::JsObject::with_null_proto();
                        for col in &schema.columns {
                            if let Some(val) = row.get(col.id) {
                                if let Ok(jv) = self.convert_oxibase_to_boa(val) {
                                    let _ = obj.set(boa_engine::JsString::from(col.name.as_str()), jv, false, context);
                                }
                            }
                        }
                        js_val = Some(boa_engine::JsValue::from(obj));
                    }
                });
            }
        });
        js_val
    }

    fn extract_new_row_json(&self, js_obj: boa_engine::object::JsObject, context: &mut boa_engine::Context) -> crate::core::Result<()> {
        let mut internal_err = None;
        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow_mut() {
                        let row = unsafe { &mut *row_ptr };
                        for col in &schema.columns {
                            if let Ok(js_val) = js_obj.get(boa_engine::JsString::from(col.name.as_str()), context) {
                                match self.convert_boa_to_oxibase(&js_val, &col.data_type, context) {
                                    Ok(v) => { let _ = row.set(col.id, v.into_coerce_to_type(col.data_type.clone())); },
                                    Err(e) => internal_err = Some(e),
                                }
                            }
                        }
                    }
                });
            }
        });
        if let Some(e) = internal_err {
            return Err(e);
        }
        Ok(())
    }

    fn convert_oxibase_to_boa(&self, value: &crate::core::Value) -> Result<boa_engine::JsValue> {
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

    fn convert_boa_to_oxibase(&self, value: &boa_engine::JsValue, dt: &crate::core::DataType, context: &mut boa_engine::Context) -> crate::core::Result<crate::core::Value> {
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
