CREATE SCHEMA IF NOT EXISTS pizza_demo;

CREATE TABLE pizza_demo.customer (
id INTEGER PRIMARY KEY AUTO_INCREMENT,
name TEXT,
address TEXT,
phone TEXT
);

CREATE TABLE pizza_demo.credit_card (
id INTEGER PRIMARY KEY AUTO_INCREMENT,
name TEXT,
number TEXT,
expiration TEXT,
UNIQUE (name, number)
);

CREATE TABLE pizza_demo.customer_credit_card (
customer_id INTEGER,
credit_card_id INTEGER
);

CREATE TABLE pizza_demo.customer_order (
id INTEGER PRIMARY KEY AUTO_INCREMENT,
customer_id INTEGER,
credit_card_id INTEGER,
total_price FLOAT
);

CREATE TABLE pizza_demo.customer_order_preference (
customer_id INTEGER,
customer_order_id INTEGER
);

CREATE TABLE pizza_demo.drinks (
id INTEGER PRIMARY KEY AUTO_INCREMENT,
name TEXT,
price FLOAT
);

CREATE TABLE pizza_demo.sizes (
id INTEGER PRIMARY KEY AUTO_INCREMENT,
name TEXT,
price FLOAT
);

CREATE TABLE pizza_demo.toppings (
id INTEGER PRIMARY KEY AUTO_INCREMENT,
name TEXT
);

CREATE TABLE pizza_demo.pizzas (
id INTEGER PRIMARY KEY AUTO_INCREMENT,
size_id INTEGER,
topping_1_id INTEGER,
topping_2_id INTEGER
);

CREATE TABLE pizza_demo.customer_pizzas (
customer_order_id INTEGER,
pizza_id INTEGER
);

CREATE TABLE pizza_demo.customer_drinks (
customer_order_id INTEGER,
drink_id INTEGER
);

INSERT INTO pizza_demo.toppings ( name ) VALUES ( 'pepperonni' ), ( 'sausage' ), ( 'ham' ), ( 'bacon' ), ( 'beef' ), ( 'chicken' ), ( 'mushroom' ), ( 'olive' ), ( 'peppers' ), ( 'onions' ), ( 'pineapple' ), ( 'jalapenos' );
INSERT INTO pizza_demo.sizes ( name, price ) VALUES ( 'S', 10.0 ), ( 'M', 12.0 ), ( 'L', 14.0 ), ( 'XL', 18.0 );
INSERT INTO pizza_demo.drinks ( name, price ) VALUES ( 'coke', 2.0 ), ( 'diet-coke', 2.0 ), ( 'red-balls', 4.0 ), ( 'liquid-schwartz', 20.0 ), ( 'extra-dimensional-water', 25.0 );

-- Insert a few dummy customers and credit cards to start
INSERT INTO pizza_demo.customer (name, address, phone) VALUES ('Alice', '123 Fake St', '555-0100'), ('Bob', '456 Null Ave', '555-0200'), ('Charlie', '789 Void Blvd', '555-0300');
INSERT INTO pizza_demo.credit_card (name, number, expiration) VALUES ('Alice', '1111222233334444', '2028-01-01'), ('Bob', '5555666677778888', '2029-05-01');
INSERT INTO pizza_demo.customer_credit_card (customer_id, credit_card_id) VALUES (1, 1), (2, 2);

-- Function to calculate the total price of an order
CREATE FUNCTION pizza_demo.calculate_order_total(order_id INTEGER) RETURNS FLOAT LANGUAGE sql AS '
    SELECT COALESCE((
        SELECT SUM(s.price)
        FROM pizza_demo.customer_pizzas cp
        JOIN pizza_demo.pizzas p ON cp.pizza_id = p.id
        JOIN pizza_demo.sizes s ON p.size_id = s.id
        WHERE cp.customer_order_id = order_id
    ), 0.0) + COALESCE((
        SELECT SUM(d.price)
        FROM pizza_demo.customer_drinks cd
        JOIN pizza_demo.drinks d ON cd.drink_id = d.id
        WHERE cd.customer_order_id = order_id
    ), 0.0);
';

-- Triggers to update the order total when items are added
CREATE TRIGGER update_total_pizza
AFTER INSERT ON pizza_demo.customer_pizzas
FOR EACH ROW LANGUAGE plsql AS '
BEGIN
    UPDATE pizza_demo.customer_order 
    SET total_price = pizza_demo.calculate_order_total(NEW.customer_order_id)
    WHERE id = NEW.customer_order_id;
END;
';

CREATE TRIGGER update_total_drink
AFTER INSERT ON pizza_demo.customer_drinks
FOR EACH ROW LANGUAGE plsql AS '
BEGIN
    UPDATE pizza_demo.customer_order 
    SET total_price = pizza_demo.calculate_order_total(NEW.customer_order_id)
    WHERE id = NEW.customer_order_id;
END;
';

-- Procedure to simulate a random order
CREATE PROCEDURE pizza_demo.simulate_random_order() LANGUAGE rhai AS '
    // Create the order
    oxibase::execute("INSERT INTO pizza_demo.customer_order (customer_id, credit_card_id, total_price) SELECT CAST(FLOOR(RANDOM() * 3) + 1 AS INTEGER), 1, 0.0");
    
    // Add random pizza
    oxibase::execute("INSERT INTO pizza_demo.pizzas (size_id, topping_1_id, topping_2_id) SELECT CAST(FLOOR(RANDOM() * 4) + 1 AS INTEGER), CAST(FLOOR(RANDOM() * 12) + 1 AS INTEGER), CAST(FLOOR(RANDOM() * 12) + 1 AS INTEGER)");
    oxibase::execute("INSERT INTO pizza_demo.customer_pizzas (customer_order_id, pizza_id) SELECT (SELECT MAX(id) FROM pizza_demo.customer_order), (SELECT MAX(id) FROM pizza_demo.pizzas)");
    
    // Add random drink
    oxibase::execute("INSERT INTO pizza_demo.customer_drinks (customer_order_id, drink_id) SELECT (SELECT MAX(id) FROM pizza_demo.customer_order), CAST(FLOOR(RANDOM() * 5) + 1 AS INTEGER)");
    
    // Log the event
    oxibase::execute("INSERT INTO system.logs (level, target, message) VALUES (''INFO'', ''pizza_demo'', ''Simulated random order'')");
';

-- Schedule the simulation
CREATE SCHEDULE simulate_pizza_orders CRON '* * * * * * *' AS 'CALL pizza_demo.simulate_random_order()';