// version 7: arrays and structs
//
// Demonstrates:
//   - fixed-size array declaration: int xs[3];
//   - array element read/write:     xs[0] = 10;  return xs[1];
//   - passing an array (decays to pointer): sum(3, xs)
//   - struct declaration and local variable
//   - field read/write:             p.x = 3;  return p.x;
//   - pointer-to-struct via ->:     ptr->x

struct Point {
    int x;
    int y;
};

struct Rect {
    int w;
    int h;
};

// Sum the first `n` elements of an int array passed as a pointer
int sum(int n, int *arr) {
    int i;
    int s;
    s = 0;
    for (i = 0; i < n; i = i + 1) {
        s = s + arr[i];
    }
    return s;
}

// Return the area of a Rect accessed through a pointer using ->
int area(struct Rect *r) {
    return r->w * r->h;
}

int main() {
    // ---- arrays ----
    int xs[5];
    xs[0] = 1;
    xs[1] = 2;
    xs[2] = 3;
    xs[3] = 4;
    xs[4] = 5;
    // sum(5, xs) == 15

    // ---- struct by value ----
    struct Point p;
    p.x = 10;
    p.y = 20;
    // p.x + p.y == 30

    // ---- pointer to struct via -> ----
    struct Rect r;
    r.w = 6;
    r.h = 7;
    struct Rect *rp;
    rp = &r;
    // area(rp) == 42

    return sum(5, xs) + p.x + p.y + area(rp);
    // expected: 15 + 10 + 20 + 42 = 87
}
