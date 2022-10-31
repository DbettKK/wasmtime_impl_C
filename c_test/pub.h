#ifndef PUB_H
#define PUB_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void* my_malloc(size_t size);
void* my_realloc(void* ptr, size_t size);
void my_free(void* ptr);

#endif // PUB_H