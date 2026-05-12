        crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
            if r.borrow().is_some() {
                if let Ok(obj) = self.build_new_row_dict(&mut context) {
                    let _ = context.register_global_property(
                        boa_engine::JsString::from("NEW"),
                        obj,
                        boa_engine::property::Attribute::all()
                    );
                }
            }
        });
        
        crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
            if r.borrow().is_some() {
                if let Ok(obj) = self.build_old_row_dict(&mut context) {
                    let _ = context.register_global_property(
                        boa_engine::JsString::from("OLD"),
                        obj,
                        boa_engine::property::Attribute::all()
                    );
                }
            }
        });

        // Add oxibase native module wrapper
        let oxibase_obj = boa_engine::object::ObjectInitializer::new(&mut context)
            .function(
                boa_engine::NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                    let sql = args
                        .first()
                        .unwrap_or(&JsValue::undefined())
                        .to_string(_ctx)
                        .unwrap_or_default()
                        .to_std_string_escaped();

                    match crate::functions::backends::execute_sql_query(&sql) {
                        Ok(res) => Ok(JsValue::from(res.rows_affected())),
                        Err(e) => Err(boa_engine::JsError::from_opaque(JsValue::from(
                            boa_engine::JsString::from(e.to_string()),
                        ))),
                    }
                }),
                boa_engine::JsString::from("execute"),
                1,
            )
            .build();

        let _ = context.register_global_property(boa_engine::JsString::from("oxibase"), oxibase_obj, boa_engine::property::Attribute::all());
