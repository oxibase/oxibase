
// --- TRIGGER CONTEXT ---
use crate::functions::backends::triggers::{CURRENT_NEW_ROW, CURRENT_OLD_ROW, CURRENT_SCHEMA};

#[derive(Clone)]
pub struct NewRowProxy;

#[derive(Clone)]
pub struct OldRowProxy;

impl NewRowProxy {
    pub fn get(&mut self, prop: &str) -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        let mut val = None;
        let mut found = false;

        CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                if let Some(idx) = schema.column_index(prop) {
                    found = true;
                    CURRENT_NEW_ROW.with(|r| {
                        if let Some(row_ptr) = *r.borrow() {
                            let row = unsafe { &*row_ptr };
                            if let Some(v) = row.get(idx) {
                                val = Some(super::rhai::value_to_dynamic(v)); // Assuming a helper exists
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

    pub fn set(&mut self, prop: &str, new_val: rhai::Dynamic) -> Result<(), Box<rhai::EvalAltResult>> {
        let mut found = false;
        let mut success = false;
        let mut error = None;

        CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                if let Some(idx) = schema.column_index(prop) {
                    found = true;
                    CURRENT_NEW_ROW.with(|r| {
                        if let Some(row_ptr) = *r.borrow_mut() {
                            let row = unsafe { &mut *row_ptr };
                            // We need to convert Dynamic back to Value
                            // We will implement dynamic_to_value later
                            match super::rhai::dynamic_to_value(new_val.clone(), schema.column(idx).data_type.clone()) {
                                Ok(v) => {
                                    row.set(idx, v);
                                    success = true;
                                }
                                Err(e) => error = Some(e.to_string()),
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
    pub fn get(&mut self, prop: &str) -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        let mut val = None;
        let mut found = false;

        CURRENT_SCHEMA.with(|s| {
            if let Some(schema_ptr) = *s.borrow() {
                let schema = unsafe { &*schema_ptr };
                if let Some(idx) = schema.column_index(prop) {
                    found = true;
                    CURRENT_OLD_ROW.with(|r| {
                        if let Some(row_ptr) = *r.borrow() {
                            let row = unsafe { &*row_ptr };
                            if let Some(v) = row.get(idx) {
                                val = Some(super::rhai::value_to_dynamic(v));
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
