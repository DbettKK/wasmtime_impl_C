#include "helper.h"

void test_memcpy(int dest, int src, size_t size) {
    int *d = transfer_i32_to_ptr(dest);
    //int *s = transfer_i32_to_ptr(src);
    int s[6];
    for (int i = 0; i < 6; i++) s[i] = 16;
    memcpy(d, s, size);
}