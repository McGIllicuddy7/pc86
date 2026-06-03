#define  UTILS_IMPLEMENTATION
#include "utils.h"
#include <stdio.h>
int main(int argc, const char ** argv){
    (void)argc;
    (void)argv;
    arena_t * arena = new_arena(); 
    string_t f = read_file_to_string(arena,"utils.h");
    string_list_t list = split_string_by(arena, f, '\n');
    for(size_t i =0; i<list.len; i++){
        printf("<" STR_FMT ">\n", STR_ARG(list.items[i]));
    }
    delete_arena(arena);
}


