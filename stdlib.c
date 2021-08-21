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

uint64_t deref(uint64_t ptr) {
    return *(uint64_t*)(ptr);
}

uint64_t equal(uint64_t x, uint64_t y) {
    return x == y;
}

uint64_t bnot(uint64_t x) {
    return ~x;
}

uint64_t not(uint64_t x) {
    return !x;
}

uint64_t and(uint64_t x, uint64_t y) {
    return x & y;
}

uint64_t or(uint64_t x, uint64_t y) {
    return x | y;
}

uint64_t xor(uint64_t x, uint64_t y) {
    return x ^ y;
}

uint64_t bsprint(uint64_t a) {
    return printf("%i", a);
}
