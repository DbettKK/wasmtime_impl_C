
#include <stdio.h>
#include <stdlib.h>

typedef struct {
    char *name;
    int age;
} Student;

extern void init_linear_memory();

extern void test_func(Student *stu);
// void test_func(Student *stu) {
//     char *new_name = "Tony";
//     stu->name = "Tony";
//     //stu->name = new_name;
//     stu->age = 31;
// }
int main() {
    init_linear_memory();
    Student *s = malloc(sizeof(Student));
    s->name = "Tom";
    s->age = 13;
    test_func(s);
    printf("guest: %s\n", s->name);
    printf("guest: %d\n", s->age);
}