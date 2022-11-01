#include "../pub.h"

extern void test_memcpy(int *dest, int *src, size_t size);

int main() {
    int *dest = malloc(sizeof(int) * 12);
    for (int i = 0; i < 12; i++) dest[i] = 3;
    int *src = malloc(sizeof(int) * 6);
    for (int i = 0; i < 12; i++) src[i] = 6;
    test_memcpy(dest, src, sizeof(int) * 6);
    //memcpy(dest, src, sizeof(int) * 6);
    for (int i = 0; i < 12; i++) printf("%d ", dest[i]);
    return 0;
}