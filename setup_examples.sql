DROP PROCEDURE IF EXISTS my_python_proc;
CREATE PROCEDURE my_python_proc(n INTEGER) LANGUAGE PYTHON AS '
def calculate_fibonacci(n):
    if n <= 0:
        return []
    elif n == 1:
        return [0]
    
    fib = [0, 1]
    for i in range(2, n):
        fib.append(fib[i-1] + fib[i-2])
    return fib

result = calculate_fibonacci(n)
print("Fibonacci sequence:", result)
';

DROP FUNCTION IF EXISTS my_python_func;
CREATE FUNCTION my_python_func(data TEXT) RETURNS TEXT LANGUAGE PYTHON AS '
import json

def process_data(data_str):
    try:
        obj = json.loads(data_str)
        if "items" in obj:
            obj["item_count"] = len(obj["items"])
        return json.dumps(obj)
    except Exception as e:
        return str(e)

process_data(data)
';
