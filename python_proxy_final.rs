    #[pyclass(module = "oxibase", name = "NewRowProxy")]
    #[derive(Debug)]
    pub struct PyNewRowProxy;

    #[pyimpl]
    impl PyNewRowProxy {
        #[pymethod(magic)]
        fn getattr(&self, attr: PyStrRef, vm: &VirtualMachine) -> PyResult<rustpython_vm::PyObjectRef> {
            let mut val = None;
            let mut found = false;
            let mut internal_err = None;

            crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
                if let Some(schema_ptr) = *s.borrow() {
                    let schema = unsafe { &*schema_ptr };
                    if let Some(idx) = schema.get_column_index(attr.as_str()) {
                        found = true;
                        crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                            if let Some(row_ptr) = *r.borrow() {
                                let row = unsafe { &*row_ptr };
                                if let Some(v) = row.get(idx) {
                                    match v {
                                        crate::core::Value::Null(_) => val = Some(vm.ctx.none()),
                                        crate::core::Value::Integer(i) => val = Some(vm.ctx.new_int(*i).into()),
                                        crate::core::Value::Float(f) => val = Some(vm.ctx.new_float(*f).into()),
                                        crate::core::Value::Text(s) => val = Some(vm.ctx.new_str(s.as_ref()).into()),
                                        crate::core::Value::Boolean(b) => val = Some(vm.ctx.new_bool(*b).into()),
                                        crate::core::Value::Timestamp(ts) => val = Some(vm.ctx.new_str(ts.to_rfc3339()).into()),
                                        crate::core::Value::Json(j) => val = Some(vm.ctx.new_str(j.as_ref()).into()),
                                    }
                                }
                            }
                        });
                    }
                }
            });

            if let Some(e) = internal_err {
                return Err(vm.new_runtime_error(e));
            }
            if !found {
                return Err(vm.new_attribute_error(format!("Column not found: {}", attr)));
            }
            Ok(val.unwrap_or_else(|| vm.ctx.none()))
        }

        #[pymethod(magic)]
        fn setattr(&self, attr: PyStrRef, value: rustpython_vm::PyObjectRef, vm: &VirtualMachine) -> PyResult<()> {
            let mut found = false;
            let mut internal_err = None;

            crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
                if let Some(schema_ptr) = *s.borrow() {
                    let schema = unsafe { &*schema_ptr };
                    if let Some(idx) = schema.get_column_index(attr.as_str()) {
                        found = true;
                        crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                            if let Some(row_ptr) = *r.borrow_mut() {
                                let row = unsafe { &mut *row_ptr };
                                if let Some(col) = schema.get_column(idx) {
                                    let converted = if value.is(&vm.ctx.none()) {
                                        Ok(crate::core::Value::Null(col.data_type.clone()))
                                    } else if let Ok(str_repr) = value.str(vm) {
                                        let s = str_repr.to_string();
                                        match col.data_type {
                                            crate::core::DataType::Integer => s.parse::<i64>().map(crate::core::Value::Integer).map_err(|_| "Failed to parse int".to_string()),
                                            crate::core::DataType::Float => s.parse::<f64>().map(crate::core::Value::Float).map_err(|_| "Failed to parse float".to_string()),
                                            crate::core::DataType::Boolean => Ok(crate::core::Value::Boolean(s == "True" || s.to_lowercase() == "true")),
                                            _ => Ok(crate::core::Value::text(s)),
                                        }
                                    } else {
                                        Err("Failed to stringify Python object".to_string())
                                    };
                                    
                                    match converted {
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
                return Err(vm.new_runtime_error(e));
            }
            if !found {
                return Err(vm.new_attribute_error(format!("Column not found: {}", attr)));
            }
            Ok(())
        }
    }

    #[pyclass(module = "oxibase", name = "OldRowProxy")]
    #[derive(Debug)]
    pub struct PyOldRowProxy;

    #[pyimpl]
    impl PyOldRowProxy {
        #[pymethod(magic)]
        fn getattr(&self, attr: PyStrRef, vm: &VirtualMachine) -> PyResult<rustpython_vm::PyObjectRef> {
            let mut val = None;
            let mut found = false;

            crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
                if let Some(schema_ptr) = *s.borrow() {
                    let schema = unsafe { &*schema_ptr };
                    if let Some(idx) = schema.get_column_index(attr.as_str()) {
                        found = true;
                        crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
                            if let Some(row_ptr) = *r.borrow() {
                                let row = unsafe { &*row_ptr };
                                if let Some(v) = row.get(idx) {
                                    match v {
                                        crate::core::Value::Null(_) => val = Some(vm.ctx.none()),
                                        crate::core::Value::Integer(i) => val = Some(vm.ctx.new_int(*i).into()),
                                        crate::core::Value::Float(f) => val = Some(vm.ctx.new_float(*f).into()),
                                        crate::core::Value::Text(s) => val = Some(vm.ctx.new_str(s.as_ref()).into()),
                                        crate::core::Value::Boolean(b) => val = Some(vm.ctx.new_bool(*b).into()),
                                        crate::core::Value::Timestamp(ts) => val = Some(vm.ctx.new_str(ts.to_rfc3339()).into()),
                                        crate::core::Value::Json(j) => val = Some(vm.ctx.new_str(j.as_ref()).into()),
                                    }
                                }
                            }
                        });
                    }
                }
            });

            if !found {
                return Err(vm.new_attribute_error(format!("Column not found: {}", attr)));
            }
            Ok(val.unwrap_or_else(|| vm.ctx.none()))
        }
    }
