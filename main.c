#define  UTILS_IMPLEMENTATION
#include "utils.h"
#include <stdio.h>
int main(int argc, const char ** argv){
    (void)argc;
    (void)argv;
    arena_t * arena = new_arena(); 
    string_t base = STR("hello world!");
    string_list_t list = new_list(arena, string_list_t);
    list_push(list, base);
    for(int i =0; i<10; i++){
        string_t tmp = arena_fmt(arena, "idx:%d", i);
        list_push(list, tmp);
    }
    for(size_t i =0; i<list.len; i++){
        printf(STR_FMT "\n", STR_ARG(list.items[i]));
    }
    delete_arena(arena);
}
