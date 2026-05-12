#[cfg(feature = "js")]
impl BoaBackend {
    fn build_new_row_dict(&self, context: &mut boa_engine::Context) -> boa_engine::JsResult<boa_engine::object::JsObject> {
        let obj = boa_engine::object::JsObject::with_object_proto(context.intrinsics());
        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow() {
                        let row = unsafe { &*row_ptr };
                        for col in &schema.columns {
                            if let Some(val) = row.get(col.id) {
                                if let Ok(js_val) = convert_oxibase_to_boa(val, context) {
                                    let _ = obj.set(boa_engine::JsString::from(col.name.as_str()), js_val, true, context);
                                }
                            }
                        }
                    }
                });
            }
        });
        Ok(obj)
    }

    fn build_old_row_dict(&self, context: &mut boa_engine::Context) -> boa_engine::JsResult<boa_engine::object::JsObject> {
        let obj = boa_engine::object::JsObject::with_object_proto(context.intrinsics());
        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow() {
                        let row = unsafe { &*row_ptr };
                        for col in &schema.columns {
                            if let Some(val) = row.get(col.id) {
                                if let Ok(js_val) = convert_oxibase_to_boa(val, context) {
                                    let _ = obj.set(boa_engine::JsString::from(col.name.as_str()), js_val, true, context);
                                }
                            }
                        }
                    }
                });
            }
        });
        Ok(obj)
    }

    fn extract_new_row_dict(&self, obj: boa_engine::object::JsObject, context: &mut boa_engine::Context) -> crate::core::Result<()> {
        let mut internal_err = None;
        crate::functions::backends::triggers::CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
                    if let Some(row_ptr) = *r.borrow_mut() {
                        let row = unsafe { &mut *row_ptr };
                        for col in &schema.columns {
                            if let Ok(js_val) = obj.get(boa_engine::JsString::from(col.name.as_str()), context) {
                                match convert_boa_to_oxibase(&js_val, &col.data_type, context) {
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
}
