// version 5: multiple functions, parameters, calls, recursion, and forward
// references. fib() is defined before main() but gcd() is defined after the
// call site, proving signatures are collected before any body is checked.
//
// fib(10) = 55, gcd(48, 36) = 12, so this returns 67
int fib(int n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

int main() {
    return fib(10) + gcd(48, 36);
}

int gcd(int a, int b) {
    while (b != 0) {
        int t = b;
        b = a % b;
        a = t;
    }
    return a;
}