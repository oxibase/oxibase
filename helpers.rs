pub(crate) fn value_to_dynamic(val: &Value) -> rhai::Dynamic {
    match val {
        Value::Integer(i) => rhai::Dynamic::from(*i),
        Value::Float(f) => rhai::Dynamic::from(*f),
        Value::Text(s) => rhai::Dynamic::from(s.as_ref().to_string()),
        Value::Boolean(b) => rhai::Dynamic::from(*b),
        Value::Null(_) => rhai::Dynamic::UNIT,
        // Fallback for others
        _ => rhai::Dynamic::from(val.to_string()),
    }
}

pub(crate) fn dynamic_to_value(val: rhai::Dynamic, dt: crate::core::DataType) -> Result<Value, crate::core::Error> {
    if val.is_unit() {
        return Ok(Value::Null(dt));
    }
    
    match dt {
        crate::core::DataType::Integer => {
            if val.is::<i64>() {
                Ok(Value::Integer(val.cast::<i64>()))
            } else if val.is::<i32>() {
                Ok(Value::Integer(val.cast::<i32>() as i64))
            } else {
                Ok(Value::Integer(val.as_int().map_err(|_| crate::core::Error::internal("Cannot cast to integer"))?))
            }
        },
        crate::core::DataType::Float => {
            if val.is::<f64>() {
                Ok(Value::Float(val.cast::<f64>()))
            } else if val.is::<f32>() {
                Ok(Value::Float(val.cast::<f32>() as f64))
            } else {
                Ok(Value::Float(val.as_float().map_err(|_| crate::core::Error::internal("Cannot cast to float"))?))
            }
        },
        crate::core::DataType::String => {
            Ok(Value::text(val.to_string()))
        },
        crate::core::DataType::Boolean => {
            if val.is::<bool>() {
                Ok(Value::Boolean(val.cast::<bool>()))
            } else {
                Ok(Value::Boolean(val.as_bool().map_err(|_| crate::core::Error::internal("Cannot cast to bool"))?))
            }
        },
        _ => {
            // Best effort fallback
            Ok(Value::text(val.to_string()))
        }
    }
}
