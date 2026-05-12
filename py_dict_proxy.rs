    fn build_new_row_dict(&self, vm: &VirtualMachine) -> Result<rustpython_vm::builtins::PyDictRef> {
        let dict = vm.ctx.new_dict();
        
        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow() {
                        let row = unsafe { &*row_ptr };
                        for col in &schema.columns {
                            if let Some(val) = row.get(col.id) {
                                if let Ok(py_val) = self.convert_oxibase_to_python(val, vm) {
                                    let _ = dict.set_item(col.name.as_str(), py_val, vm);
                                }
                            }
                        }
                    }
                });
            }
        });
        
        Ok(dict)
    }

    fn build_old_row_dict(&self, vm: &VirtualMachine) -> Result<rustpython_vm::builtins::PyDictRef> {
        let dict = vm.ctx.new_dict();
        
        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow() {
                        let row = unsafe { &*row_ptr };
                        for col in &schema.columns {
                            if let Some(val) = row.get(col.id) {
                                if let Ok(py_val) = self.convert_oxibase_to_python(val, vm) {
                                    let _ = dict.set_item(col.name.as_str(), py_val, vm);
                                }
                            }
                        }
                    }
                });
            }
        });
        
        Ok(dict)
    }

    fn extract_new_row_dict(&self, dict: rustpython_vm::builtins::PyDictRef, vm: &VirtualMachine) -> Result<()> {
        let mut internal_err = None;
        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow_mut() {
                        let row = unsafe { &mut *row_ptr };
                        for col in &schema.columns {
                            if let Ok(py_val) = dict.get_item(col.name.as_str(), vm) {
                                match self.convert_python_to_oxibase(&py_val, vm) {
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
