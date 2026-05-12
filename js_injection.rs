        crate::functions::backends::triggers::CURRENT_NEW_ROW.with(|r| {
            if r.borrow().is_some() {
                let target = boa_engine::object::JsObject::with_null_proto();
                if let Ok(proxy) = boa_engine::object::builtins::JsProxy::builder(target)
                    .get(js_row_get)
                    .set(js_row_set)
                    .build(&mut context) 
                {
                    let _ = context.register_global_property(
                        JsString::from("NEW"),
                        proxy,
                        boa_engine::property::Attribute::all()
                    );
                }
            }
        });
        
        crate::functions::backends::triggers::CURRENT_OLD_ROW.with(|r| {
            if r.borrow().is_some() {
                let target = boa_engine::object::JsObject::with_null_proto();
                if let Ok(proxy) = boa_engine::object::builtins::JsProxy::builder(target)
                    .get(js_row_get)
                    .build(&mut context) 
                {
                    let _ = context.register_global_property(
                        JsString::from("OLD"),
                        proxy,
                        boa_engine::property::Attribute::all()
                    );
                }
            }
        });
        
        // Add oxibase native module wrapper
        let oxibase_obj = boa_engine::object::JsObject::with_null_proto();
        let exec_fn = boa_engine::object::FunctionObjectBuilder::new(
            context.realm(),
            boa_engine::NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let sql = args.get(0).unwrap_or(&JsValue::undefined()).to_string(ctx)?;
                match crate::functions::backends::execute_sql_query(&sql.to_std_string_escaped()) {
                    Ok(res) => Ok(JsValue::new(res.rows_affected() as i32)),
                    Err(e) => Err(boa_engine::JsError::from_opaque(JsValue::new(JsString::from(e.to_string())))),
                }
            }),
        )
        .name("execute")
        .length(1)
        .build();
        let _ = oxibase_obj.set("execute", exec_fn, false, &mut context);
        let _ = context.register_global_property(
            JsString::from("oxibase"),
            oxibase_obj,
            boa_engine::property::Attribute::all()
        );
