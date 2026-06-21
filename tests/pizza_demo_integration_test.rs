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

// Simple script splitter that avoids splitting inside string literals ('...') or dollar quotes ($$...$$)
fn parse_sql_script(script: &str) -> Vec<String> {
    let mut queries = Vec::new();
    let mut current_query = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_dollar_quote = false;

    let chars: Vec<char> = script.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if in_single_quote {
            if c == '\'' {
                if i + 1 < chars.len() && chars[i + 1] == '\'' {
                    current_query.push(c);
                    current_query.push(chars[i + 1]);
                    i += 2;
                    continue;
                } else {
                    in_single_quote = false;
                }
            }
        } else if in_double_quote {
            if c == '"' {
                if i + 1 < chars.len() && chars[i + 1] == '"' {
                    current_query.push(c);
                    current_query.push(chars[i + 1]);
                    i += 2;
                    continue;
                } else {
                    in_double_quote = false;
                }
            }
        } else if in_dollar_quote {
            if c == '$' && i + 1 < chars.len() && chars[i + 1] == '$' {
                current_query.push(c);
                current_query.push(chars[i + 1]);
                in_dollar_quote = false;
                i += 2;
                continue;
            }
        } else {
            if c == '\'' {
                in_single_quote = true;
            } else if c == '"' {
                in_double_quote = true;
            } else if c == '$' && i + 1 < chars.len() && chars[i + 1] == '$' {
                in_dollar_quote = true;
                current_query.push(c);
                current_query.push(chars[i + 1]);
                i += 2;
                continue;
            } else if c == ';' {
                queries.push(current_query.trim().to_string());
                current_query.clear();
                i += 1;
                continue;
            }
        }

        current_query.push(c);
        i += 1;
    }

    if !current_query.trim().is_empty() {
        queries.push(current_query.trim().to_string());
    }

    queries
}

#[test]
fn test_pizza_demo_setup_and_execution() {
    let db = Database::open("memory://pizza_demo_test").unwrap();

    let script = include_str!("../src/bin/workspace/templates/pizza_demo.sql");
    let queries = parse_sql_script(script);

    for q in queries {
        if !q.trim().is_empty() {
            db.execute(&q, ())
                .unwrap_or_else(|e| panic!("Failed to execute query: {}\nError: {:?}", q, e));
        }
    }

    // Call stored procedure to simulate an order
    db.execute("CALL pizza_tx.simulate_random_order()", ())
        .unwrap();

    // Re-run procedurals to sync daily analytical summary
    db.execute("CALL pizza_analytics.sync_daily_summary()", ())
        .unwrap();

    // Validating Real-time Trigger Event Replication across schemas
    let log_count: i64 = db
        .query_one("SELECT COUNT(*) FROM pizza_analytics.order_events_log", ())
        .unwrap();
    assert_eq!(
        log_count, 1,
        "Order should be replicated to analytics schema"
    );

    // Validating PL/SQL syncer procedure
    let summary_count: i64 = db
        .query_one(
            "SELECT COUNT(*) FROM pizza_analytics.daily_sales_summary",
            (),
        )
        .unwrap();
    assert_eq!(summary_count, 1, "Daily summary should be synced");

    // Validating standard reporting view incorporating Rhai UDFs
    let details_count: i64 = db
        .query_one("SELECT COUNT(*) FROM pizza_analytics.v_order_details", ())
        .unwrap();
    assert_eq!(
        details_count, 1,
        "v_order_details view should return simulated order"
    );

    // Validating Window Functions + CTE analytical view
    let clv_count: i64 = db
        .query_one(
            "SELECT COUNT(*) FROM pizza_analytics.v_customer_lifetime_value WHERE name = 'Alice' OR name = 'Bob' OR name = 'Charlie'",
            (),
        )
        .unwrap();
    assert_eq!(
        clv_count, 3,
        "All seeded customers should be ranked and queried"
    );

    // Validating ROLLUP analytical view
    let rollup_count: i64 = db
        .query_one(
            "SELECT COUNT(*) FROM pizza_analytics.v_revenue_by_size_and_topping",
            (),
        )
        .unwrap();
    assert!(
        rollup_count > 0,
        "ROLLUP view should return rows and subtotals"
    );
}
