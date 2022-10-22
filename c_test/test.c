#include <stdio.h>
#include <stdlib.h>

__attribute__((export_name("realloc")))
void* my_realloc(void *ptr, size_t size) { return realloc(ptr, size); }

__attribute__((export_name("malloc")))
void* my_malloc(size_t size) { return malloc(size); }

__attribute__((export_name("free")))
void my_free(void *ptr) { free(ptr); }

extern int* test_func(int *ptr, int size);

int main() {
    int *p = malloc(sizeof(int) * 6);
    printf("guest: old_ptr: %p\n", p);
    p[1] = 10;
    p = test_func(p, 2);
    printf("guest: new_ptr: %p\n", p);
    printf("%d\n", p[1]);
    return 0;
}