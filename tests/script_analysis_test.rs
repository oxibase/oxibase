// Copyright 2026 Oxibase Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use oxibase::api::Database;

#[test]
fn test_integration_rhai_analysis() {
    let db = Database::open_in_memory().unwrap();

    let script = r#"
        oxibase::execute("INSERT INTO pizza_demo.pizzas (size_id, topping_id) VALUES (1, 2)");
        if oxibase::query("SELECT COUNT(*) FROM pizza_demo.customer_order").get(0) > 0 {
            oxibase::execute("CALL pizza_demo.simulate_random_order()");
        }
    "#;

    let results = db.analyze_script(script, "rhai").unwrap();

    let tables: Vec<&str> = results
        .iter()
        .filter(|o| o.object_type == "Table")
        .map(|o| o.name.as_str())
        .collect();
    let procedures: Vec<&str> = results
        .iter()
        .filter(|o| o.object_type == "Procedure")
        .map(|o| o.name.as_str())
        .collect();

    assert!(tables.contains(&"pizza_demo.pizzas"));
    assert!(tables.contains(&"pizza_demo.customer_order"));
    assert!(procedures.contains(&"pizza_demo.simulate_random_order"));
}

#[test]
fn test_integration_rhai_dynamic_analysis() {
    let db = Database::open_in_memory().unwrap();

    let script = r#"
        let base = "SELECT * FROM ";
        let table = "users";
        oxibase::query(base + table);
    "#;

    let results = db.analyze_script(script, "rhai").unwrap();
    assert!(results.iter().any(|o| o.object_type == "Dynamic"));
}

#[test]
#[cfg(feature = "python")]
fn test_integration_python_analysis() {
    let db = Database::open_in_memory().unwrap();

    let script = r#"
import oxibase
oxibase.execute("UPDATE inventory SET qty = qty - 1 WHERE item_id = 5")
if True:
    oxibase.query("SELECT * FROM suppliers")
"#;

    let results = db.analyze_script(script, "python").unwrap();

    let tables: Vec<&str> = results
        .iter()
        .filter(|o| o.object_type == "Table")
        .map(|o| o.name.as_str())
        .collect();

    assert!(tables.contains(&"inventory"));
    assert!(tables.contains(&"suppliers"));
}

#[test]
#[cfg(feature = "python")]
fn test_integration_python_dynamic_analysis() {
    let db = Database::open_in_memory().unwrap();

    let script = r#"
import oxibase
tbl = "logs"
oxibase.execute("SELECT * FROM " + tbl)
"#;

    let results = db.analyze_script(script, "python").unwrap();
    assert!(results.iter().any(|o| o.object_type == "Dynamic"));
}

#[test]
fn test_integration_plsql_analysis() {
    let db = Database::open_in_memory().unwrap();

    let script = r#"
        SELECT * FROM active_users JOIN logins ON active_users.id = logins.user_id;
        CALL audit_login();
    "#;

    let results = db.analyze_script(script, "plsql").unwrap();

    let tables: Vec<&str> = results
        .iter()
        .filter(|o| o.object_type == "Table")
        .map(|o| o.name.as_str())
        .collect();
    let procedures: Vec<&str> = results
        .iter()
        .filter(|o| o.object_type == "Procedure")
        .map(|o| o.name.as_str())
        .collect();

    assert!(tables.contains(&"active_users"));
    assert!(tables.contains(&"logins"));
    assert!(procedures.contains(&"audit_login"));
}
