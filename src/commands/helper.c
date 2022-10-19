#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void* my_realloc(void *ptr, size_t size) {
    return realloc(ptr, size);
}

void test_func(int *ptr, int size) {
    int *new_ptr;
    new_ptr = my_realloc(ptr, size);
    ptr = new_ptr;
    printf("%d\n", ptr[1]);
}