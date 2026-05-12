// --- TRIGGER CONTEXT ---

#[derive(Clone)]
pub struct NewRowProxy;

#[derive(Clone)]
pub struct OldRowProxy;

impl NewRowProxy {
    pub fn get(&mut self, prop: &str) -> std::result::Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        let mut val = None;
        let mut found = false;

        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                if let Some(idx) = schema.get_column_index(prop) {
                    found = true;
                    crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                        if let Some(row_ptr) = *r.borrow() {
                            let row = unsafe { &*row_ptr };
                            if let Some(v) = row.get(idx) {
                                val = Some(crate::functions::backends::rhai::value_to_dynamic(v));
                            }
                        }
                    });
                }
            }
        });

        if !found {
            return Err(format!("Column not found: {}", prop).into());
        }
        
        Ok(val.unwrap_or(rhai::Dynamic::UNIT))
    }

    pub fn set(&mut self, prop: &str, new_val: rhai::Dynamic) -> std::result::Result<(), Box<rhai::EvalAltResult>> {
        let mut found = false;
        let mut success = false;
        let mut error = None;

        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                if let Some(idx) = schema.get_column_index(prop) {
                    found = true;
                    crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                        if let Some(row_ptr) = *r.borrow_mut() {
                            let row = unsafe { &mut *row_ptr };
                            if let Some(col) = schema.get_column(idx) {
                                match crate::functions::backends::rhai::dynamic_to_value(new_val.clone(), col.data_type.clone()) {
                                    Ok(v) => {
                                        row.set(idx, v);
                                        success = true;
                                    }
                                    Err(e) => error = Some(e.to_string()),
                                }
                            }
                        }
                    });
                }
            }
        });

        if !found {
            return Err(format!("Column not found: {}", prop).into());
        }
        if let Some(err) = error {
            return Err(err.into());
        }
        
        Ok(())
    }
}

impl OldRowProxy {
    pub fn get(&mut self, prop: &str) -> std::result::Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        let mut val = None;
        let mut found = false;

        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                if let Some(idx) = schema.get_column_index(prop) {
                    found = true;
                    crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
                        if let Some(row_ptr) = *r.borrow() {
                            let row = unsafe { &*row_ptr };
                            if let Some(v) = row.get(idx) {
                                val = Some(crate::functions::backends::rhai::value_to_dynamic(v));
                            }
                        }
                    });
                }
            }
        });

        if !found {
            return Err(format!("Column not found: {}", prop).into());
        }
        
        Ok(val.unwrap_or(rhai::Dynamic::UNIT))
    }
}

pub(crate) fn value_to_dynamic(val: &crate::core::Value) -> rhai::Dynamic {
    match val {
        crate::core::Value::Integer(i) => rhai::Dynamic::from(*i),
        crate::core::Value::Float(f) => rhai::Dynamic::from(*f),
        crate::core::Value::Text(s) => rhai::Dynamic::from(s.as_ref().to_string()),
        crate::core::Value::Boolean(b) => rhai::Dynamic::from(*b),
        crate::core::Value::Null(_) => rhai::Dynamic::UNIT,
        _ => rhai::Dynamic::from(val.to_string()),
    }
}

pub(crate) fn dynamic_to_value(val: rhai::Dynamic, dt: crate::core::DataType) -> std::result::Result<crate::core::Value, crate::core::Error> {
    if val.is_unit() {
        return Ok(crate::core::Value::Null(dt));
    }
    
    match dt {
        crate::core::DataType::Integer => {
            if val.is::<i64>() {
                Ok(crate::core::Value::Integer(val.cast::<i64>()))
            } else if val.is::<i32>() {
                Ok(crate::core::Value::Integer(val.cast::<i32>() as i64))
            } else {
                Ok(crate::core::Value::Integer(val.as_int().map_err(|_| crate::core::Error::internal("Cannot cast to integer"))?))
            }
        },
        crate::core::DataType::Float => {
            if val.is::<f64>() {
                Ok(crate::core::Value::Float(val.cast::<f64>()))
            } else if val.is::<f32>() {
                Ok(crate::core::Value::Float(val.cast::<f32>() as f64))
            } else {
                Ok(crate::core::Value::Float(val.as_float().map_err(|_| crate::core::Error::internal("Cannot cast to float"))?))
            }
        },
        crate::core::DataType::Text => {
            Ok(crate::core::Value::text(val.to_string()))
        },
        crate::core::DataType::Boolean => {
            if val.is::<bool>() {
                Ok(crate::core::Value::Boolean(val.cast::<bool>()))
            } else {
                Ok(crate::core::Value::Boolean(val.as_bool().map_err(|_| crate::core::Error::internal("Cannot cast to bool"))?))
            }
        },
        _ => {
            Ok(crate::core::Value::text(val.to_string()))
        }
    }
}
