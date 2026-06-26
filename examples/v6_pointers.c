// version 6: pointers and address-of.
//
// Demonstrates:
//   &x       - take the address of a local variable
//   *p       - dereference a pointer (read)
//   *p = v   - dereference a pointer (write)
//   int **pp - pointer to a pointer
//
// Expected return value: 42

int main() {
    // basic address-of and deref
    int x = 7;
    int *p = &x;
    *p = 10;        // x is now 10

    // pointer-to-pointer
    int y = 5;
    int *q = &y;
    int **pp = &q;
    **pp = 32;      // y is now 32

    // reads through pointers
    return *p + *q; // 10 + 32 = 42
}
