#include <stdio.h>
#include <stdlib.h>

typedef struct {
    char *name;     // 8byte
    int color : 4;  // 1byte
    int color2;     // 4byte
    char age;       // 1byte
} BitFields;

int main() {
    printf("sizeof(BitFileds) = %lu\n", sizeof(BitFields));
    printf("%lu\n", __offsetof(BitFields, color2));
}