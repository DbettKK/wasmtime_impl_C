#include "../pub.h"
// 结构体变量(非指针)是直接一起存的
typedef struct {
    char *name;
    int age;
} Stu;

typedef struct {
    char *name;
    int age;
} Tea;

typedef struct {
    Stu st;
    Tea te;
} Class;

extern void check_struct(void *c);

int main() {
    Class *c = malloc(sizeof(Class));
    Stu s = {.name = "student", .age = 21 };
    Tea t = {.name = "teacher", .age = 33 };
    c->st = s;
    c->te = t;
    check_struct(c);
}