#include <stdint.h>
#include <stdio.h>

uint64_t add(uint64_t a, uint64_t b) {
    return a + b;
}

uint64_t sub(uint64_t a, uint64_t b) {
    return a - b;
}

uint64_t less(uint64_t a, uint64_t b) {
    return a < b;
}

uint64_t bsprint(uint64_t a) {
    printf("%i", a);
}
