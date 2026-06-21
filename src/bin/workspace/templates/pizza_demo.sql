CREATE SCHEMA IF NOT EXISTS pizza_tx;
CREATE SCHEMA IF NOT EXISTS pizza_analytics;

-- -------------------------------------------------------------
-- TRANSACTIONAL TABLES (OLTP - Schema: pizza_tx)
-- -------------------------------------------------------------
CREATE TABLE pizza_tx.customer (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT,
    address TEXT,
    phone TEXT
);

CREATE TABLE pizza_tx.credit_card (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT,
    number TEXT,
    expiration TEXT,
    UNIQUE (name, number)
);

CREATE TABLE pizza_tx.customer_credit_card (
    customer_id INTEGER,
    credit_card_id INTEGER,
    FOREIGN KEY (customer_id) REFERENCES pizza_tx.customer(id) ON DELETE CASCADE,
    FOREIGN KEY (credit_card_id) REFERENCES pizza_tx.credit_card(id) ON DELETE CASCADE
);

CREATE TABLE pizza_tx.customer_order (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    customer_id INTEGER,
    credit_card_id INTEGER,
    total_price FLOAT,
    FOREIGN KEY (customer_id) REFERENCES pizza_tx.customer(id) ON DELETE CASCADE,
    FOREIGN KEY (credit_card_id) REFERENCES pizza_tx.credit_card(id) ON DELETE SET NULL
);

CREATE TABLE pizza_tx.drinks (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT,
    price FLOAT
);

CREATE TABLE pizza_tx.sizes (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT,
    price FLOAT
);

CREATE TABLE pizza_tx.toppings (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT
);

CREATE TABLE pizza_tx.pizzas (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    size_id INTEGER,
    topping_1_id INTEGER,
    topping_2_id INTEGER,
    FOREIGN KEY (size_id) REFERENCES pizza_tx.sizes(id) ON DELETE RESTRICT,
    FOREIGN KEY (topping_1_id) REFERENCES pizza_tx.toppings(id) ON DELETE RESTRICT,
    FOREIGN KEY (topping_2_id) REFERENCES pizza_tx.toppings(id) ON DELETE RESTRICT
);

CREATE TABLE pizza_tx.customer_pizzas (
    customer_order_id INTEGER,
    pizza_id INTEGER,
    FOREIGN KEY (customer_order_id) REFERENCES pizza_tx.customer_order(id) ON DELETE CASCADE,
    FOREIGN KEY (pizza_id) REFERENCES pizza_tx.pizzas(id) ON DELETE CASCADE
);

CREATE TABLE pizza_tx.customer_drinks (
    customer_order_id INTEGER,
    drink_id INTEGER,
    FOREIGN KEY (customer_order_id) REFERENCES pizza_tx.customer_order(id) ON DELETE CASCADE,
    FOREIGN KEY (drink_id) REFERENCES pizza_tx.drinks(id) ON DELETE CASCADE
);

-- -------------------------------------------------------------
-- DATA SEEDING
-- -------------------------------------------------------------
INSERT INTO pizza_tx.toppings ( name ) VALUES ( 'pepperonni' ), ( 'sausage' ), ( 'ham' ), ( 'bacon' ), ( 'beef' ), ( 'chicken' ), ( 'mushroom' ), ( 'olive' ), ( 'peppers' ), ( 'onions' ), ( 'pineapple' ), ( 'jalapenos' );
INSERT INTO pizza_tx.sizes ( name, price ) VALUES ( 'S', 10.0 ), ( 'M', 12.0 ), ( 'L', 14.0 ), ( 'XL', 18.0 );
INSERT INTO pizza_tx.drinks ( name, price ) VALUES ( 'coke', 2.0 ), ( 'diet-coke', 2.0 ), ( 'red-balls', 4.0 ), ( 'liquid-schwartz', 20.0 ), ( 'extra-dimensional-water', 25.0 );

INSERT INTO pizza_tx.customer (name, address, phone) VALUES 
('Alice', '123 Fake St', '555-0100'), 
('Bob', '456 Null Ave', '555-0200'), 
('Charlie', '789 Void Blvd', '555-0300');

INSERT INTO pizza_tx.credit_card (name, number, expiration) VALUES 
('Alice', '1111222233334444', '2028-01-01'), 
('Bob', '5555666677778888', '2029-05-01');

INSERT INTO pizza_tx.customer_credit_card (customer_id, credit_card_id) VALUES (1, 1), (2, 2);

-- -------------------------------------------------------------
-- ANALYTICAL TABLES (OLAP - Schema: pizza_analytics)
-- -------------------------------------------------------------
CREATE TABLE pizza_analytics.order_events_log (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    order_id INTEGER,
    customer_id INTEGER,
    total_price FLOAT,
    replicated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE pizza_analytics.daily_sales_summary (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    summary_date TEXT,
    total_orders INTEGER,
    total_revenue FLOAT,
    UNIQUE(summary_date)
);

-- -------------------------------------------------------------
-- USER-DEFINED FUNCTIONS
-- -------------------------------------------------------------
CREATE FUNCTION pizza_analytics.categorize_revenue(total FLOAT) RETURNS TEXT 
LANGUAGE rhai AS '
    if total < 15.0 {
        return "BRONZE";
    } else if total < 30.0 {
        return "SILVER";
    } else {
        return "GOLD";
    }
';

CREATE FUNCTION pizza_analytics.free_delivery_eligible(total FLOAT) RETURNS BOOLEAN 
LANGUAGE rhai AS '
    return total >= 20.0;
';

-- -------------------------------------------------------------
-- PROCEDURAL TRIGGERS
-- -------------------------------------------------------------
CREATE TRIGGER validate_customer_phone
    BEFORE INSERT ON pizza_tx.customer
    FOR EACH ROW
    LANGUAGE rhai
AS '
    if oxibase.ctx.new.phone == "" {
        throw "Phone number is mandatory";
    }
    let phone_str = oxibase.ctx.new.phone;
    if !phone_str.contains("555-") {
        throw "Invalid phone format. Must be a standard 555- exchange.";
    }
';

CREATE TRIGGER replicate_order_event
    AFTER INSERT ON pizza_tx.customer_order
    FOR EACH ROW
    LANGUAGE rhai
AS '
    let order_id = oxibase.ctx.new.id;
    let cust_id = oxibase.ctx.new.customer_id;
    let price = oxibase.ctx.new.total_price;
    let query = "INSERT INTO pizza_analytics.order_events_log (order_id, customer_id, total_price) VALUES (" + to_string(order_id) + ", " + to_string(cust_id) + ", " + to_string(price) + ")";
    oxibase::execute(query);
';

-- -------------------------------------------------------------
-- STORED PROCEDURES
-- -------------------------------------------------------------
CREATE PROCEDURE pizza_analytics.sync_daily_summary()
LANGUAGE plsql
AS $$
BEGIN
    DELETE FROM pizza_analytics.daily_sales_summary WHERE summary_date = 'TODAY';
    
    INSERT INTO pizza_analytics.daily_sales_summary (summary_date, total_orders, total_revenue)
    SELECT 
        'TODAY', 
        COUNT(*), 
        COALESCE(SUM(total_price), 0.0) 
    FROM pizza_tx.customer_order;
END;
$$;

CREATE PROCEDURE pizza_tx.simulate_random_order() 
LANGUAGE rhai 
AS '
    oxibase::execute("INSERT INTO pizza_tx.customer_order (customer_id, credit_card_id, total_price) SELECT CAST(FLOOR(RANDOM() * 3) + 1 AS INTEGER), 1, 0.0");

    oxibase::execute("INSERT INTO pizza_tx.pizzas (size_id, topping_1_id, topping_2_id) SELECT CAST(FLOOR(RANDOM() * 4) + 1 AS INTEGER), CAST(FLOOR(RANDOM() * 12) + 1 AS INTEGER), CAST(FLOOR(RANDOM() * 12) + 1 AS INTEGER)");
    oxibase::execute("INSERT INTO pizza_tx.customer_pizzas (customer_order_id, pizza_id) SELECT (SELECT MAX(id) FROM pizza_tx.customer_order), (SELECT MAX(id) FROM pizza_tx.pizzas)");

    oxibase::execute("INSERT INTO pizza_tx.customer_drinks (customer_order_id, drink_id) SELECT (SELECT MAX(id) FROM pizza_tx.customer_order), CAST(FLOOR(RANDOM() * 5) + 1 AS INTEGER)");

    oxibase::execute("UPDATE pizza_tx.customer_order SET total_price = COALESCE((SELECT SUM(s.price) FROM pizza_tx.customer_pizzas cp JOIN pizza_tx.pizzas p ON cp.pizza_id = p.id JOIN pizza_tx.sizes s ON p.size_id = s.id WHERE cp.customer_order_id = (SELECT MAX(id) FROM pizza_tx.customer_order)), 0.0) + COALESCE((SELECT SUM(d.price) FROM pizza_tx.customer_drinks cd JOIN pizza_tx.drinks d ON cd.drink_id = d.id WHERE cd.customer_order_id = (SELECT MAX(id) FROM pizza_tx.customer_order)), 0.0) WHERE id = (SELECT MAX(id) FROM pizza_tx.customer_order)");

    oxibase::log("INFO", "Simulated transactional pizza order successfully created.");
';

-- -------------------------------------------------------------
-- AUTOMATION & CRON TASK SCHEDULING
-- -------------------------------------------------------------
CREATE SCHEDULE simulate_pizza_orders 
CRON '0 * * * * * *' 
AS 'CALL pizza_tx.simulate_random_order()';

CREATE SCHEDULE sync_analytics_daily 
CRON '30 * * * * * *' 
AS 'CALL pizza_analytics.sync_daily_summary()';

-- -------------------------------------------------------------
-- REPORTING & BUSINESS INTELLIGENCE VIEWS (pizza_analytics)
-- -------------------------------------------------------------
-- 1. Standard View utilizing custom functions
CREATE VIEW pizza_analytics.v_order_details AS
SELECT 
    co.id AS order_id,
    c.name AS customer_name,
    co.total_price,
    pizza_analytics.free_delivery_eligible(co.total_price) AS free_delivery,
    pizza_analytics.categorize_revenue(co.total_price) AS customer_tier
FROM pizza_tx.customer_order co
JOIN pizza_tx.customer c ON co.customer_id = c.id;

-- 2. RFM / CLV analysis using CTEs and Window Functions (RANK() OVER, SUM() OVER)
CREATE VIEW pizza_analytics.v_customer_lifetime_value AS
WITH customer_totals AS (
    SELECT 
        customer_id,
        COUNT(id) AS order_count,
        SUM(total_price) AS total_spend
    FROM pizza_tx.customer_order
    GROUP BY customer_id
)
SELECT 
    c.id,
    c.name,
    ct.order_count,
    ct.total_spend,
    RANK() OVER (ORDER BY ct.total_spend DESC) AS revenue_rank,
    SUM(ct.total_spend) OVER (ORDER BY ct.total_spend DESC) AS cumulative_sales
FROM pizza_tx.customer c
LEFT JOIN customer_totals ct ON c.id = ct.customer_id;

-- 3. Size and topping preferences using ROLLUP
CREATE VIEW pizza_analytics.v_revenue_by_size_and_topping AS
SELECT 
    s.name AS size_name,
    t.name AS topping_name,
    COUNT(co.id) AS total_orders,
    SUM(co.total_price) AS total_revenue
FROM pizza_tx.customer_order co
JOIN pizza_tx.customer_pizzas cp ON co.id = cp.customer_order_id
JOIN pizza_tx.pizzas p ON cp.pizza_id = p.id
JOIN pizza_tx.sizes s ON p.size_id = s.id
JOIN pizza_tx.toppings t ON p.topping_1_id = t.id
GROUP BY ROLLUP(s.name, t.name);
