DROP PROCEDURE IF EXISTS my_python_proc;
CREATE PROCEDURE my_python_proc(n INTEGER) LANGUAGE PYTHON AS '
def calculate_fibonacci(n):
    if n is None:
        n = 10
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
