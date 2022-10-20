#include <stdio.h>
#include <stdlib.h>
#include <string.h>


int* test_func(int *ptr, int size) {
    // int *new_ptr;
    // new_ptr = realloc(ptr, size);
    // ptr = new_ptr;
    printf("%d\n", ptr[1]);
    return ptr;
}