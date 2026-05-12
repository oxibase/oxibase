        crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
            if r.borrow().is_some() {
                let target = boa_engine::object::JsObject::with_null_proto();
                let proxy = boa_engine::object::builtins::JsProxy::new(target, boa_engine::object::JsObject::with_null_proto(), &mut context).unwrap();
                let get_fn = boa_engine::object::FunctionObjectBuilder::new(context.realm(), boa_engine::NativeFunction::from_fn_ptr(js_row_get)).build();
                let set_fn = boa_engine::object::FunctionObjectBuilder::new(context.realm(), boa_engine::NativeFunction::from_fn_ptr(js_row_set)).build();
                let handler = proxy.handler();
                let _ = handler.set(boa_engine::JsString::from("get"), get_fn, true, &mut context);
                let _ = handler.set(boa_engine::JsString::from("set"), set_fn, true, &mut context);

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
                let proxy = boa_engine::object::builtins::JsProxy::new(target, boa_engine::object::JsObject::with_null_proto(), &mut context).unwrap();
                let get_fn = boa_engine::object::FunctionObjectBuilder::new(context.realm(), boa_engine::NativeFunction::from_fn_ptr(js_row_get)).build();
                let handler = proxy.handler();
                let _ = handler.set(boa_engine::JsString::from("get"), get_fn, true, &mut context);

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

        let _ = context.register_global_property(boa_engine::JsString::from("oxibase"), oxibase_obj, boa_engine::property::Attribute::all());
