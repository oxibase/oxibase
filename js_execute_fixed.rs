        crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
            if r.borrow().is_some() {
                let target = boa_engine::object::JsObject::with_null_proto();
                let proxy = boa_engine::object::builtins::JsProxy::builder(target)
                    .get(js_row_get)
                    .set(js_row_set)
                    .build(&mut context);
                
                let _ = context.register_global_property(
                    boa_engine::JsString::from("NEW"),
                    proxy,
                    boa_engine::property::Attribute::all()
                );
            }
        });
        
        crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
            if r.borrow().is_some() {
                let target = boa_engine::object::JsObject::with_null_proto();
                let proxy = boa_engine::object::builtins::JsProxy::builder(target)
                    .get(js_row_get)
                    .build(&mut context);
                    
                let _ = context.register_global_property(
                    boa_engine::JsString::from("OLD"),
                    proxy,
                    boa_engine::property::Attribute::all()
                );
            }
        });
        
        // Ensure oxibase execute is registered directly
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

        let _ = context.register_global_property(boa_engine::JsString::from("oxibase"), oxibase_obj, Default::default());
