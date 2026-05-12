use boa_engine::{Context, JsValue, object::builtins::JsProxy, JsResult, JsString};
use boa_engine::object::JsObject;

fn proxy_get(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get(1).unwrap_or(&JsValue::undefined()).to_string(context)?;
    println!("Intercepted GET for property: {}", key.to_std_string_escaped());
    
    if key.to_std_string_escaped() == "my_dynamic_prop" {
        return Ok(JsValue::new(42));
    }
    
    Ok(JsValue::undefined())
}

fn proxy_set(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let key = args.get(1).unwrap_or(&JsValue::undefined()).to_string(context)?;
    let undefined = JsValue::undefined();
    let val = args.get(2).unwrap_or(&undefined);
    
    println!("Intercepted SET for property: {} to {}", key.to_std_string_escaped(), val.display());
    
    Ok(JsValue::new(true))
}

fn main() {
    let mut context = Context::default();
    
    let target = JsObject::with_null_proto();
    let proxy = JsProxy::builder(target)
        .get(proxy_get)
        .set(proxy_set)
        .build(&mut context);
        
    context.register_global_property(JsString::from("NEW"), proxy, boa_engine::property::Attribute::all()).unwrap();
    
    let src = boa_engine::Source::from_bytes(r#"
        let x = NEW.my_dynamic_prop;
        NEW.some_other_prop = "hello world";
        x
    "#);
    
    let res = context.eval(src).unwrap();
    println!("Result: {}", res.display());
}
