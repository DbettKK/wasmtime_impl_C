#include "pub.h"

__attribute__((export_name("malloc")))
void* my_malloc(size_t size) { return malloc(size); }
__attribute__((export_name("realloc")))
void* my_realloc(void* ptr, size_t size) { return realloc(ptr, size); }
__attribute__((export_name("free")))
void my_free(void* ptr) { free(ptr); }
