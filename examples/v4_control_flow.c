//version 4: added this to test all the features I added to make sure everything works
//After testing, everything seems to work
int main() {
    int sum = 0;

    for (int i = 1; i <= 10; i = i + 1) {
        if (i % 2 == 0 || i % 3 == 0) {
            sum = sum + i;
        }
    }

    int n = sum;
    while (n > 100 && n != 0) {
        n = n - 100;
    }

    return n;
}