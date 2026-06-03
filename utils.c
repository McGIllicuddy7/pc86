#include "utils.h"
#include <string.h>
#include <stdarg.h>
#include <stdio.h>
arena_t * new_arena(void){
    const size_t BYTE_COUNT = 4096*4096;
    arena_t * out = malloc(sizeof(arena_t) +(64-64%sizeof(arena_t))+BYTE_COUNT);
    out->bytes = (unsigned char *)out+(sizeof(arena_t) +(64-64%sizeof(arena_t)));
    out->last_ptr = 0;
    out->bump_offset = 0;
    out->length = BYTE_COUNT;
    out->next = 0;
    return out;
}
void delete_arena(arena_t * arena){
    if(arena->next){
        delete_arena(arena->next);
    }
    free(arena);
}
void * arena_alloc(arena_t * arena, size_t count){
    void * out;
    size_t min_count;
    if(count%16 != 0){
        count += 16-count%16;
    }
    if (arena->bump_offset+count>= arena->length){
        if (arena->next){
            return arena_alloc(arena->next, count);
        }else{
            min_count = arena->length;
            while(min_count<count){
                min_count *= 2;
            }         
            arena->next = new_arena_sized(min_count);
            return arena_alloc(arena->next, count);
        }
    }
    out = arena->bytes+arena->bump_offset;
    arena->bump_offset += count;
    arena->last_ptr = out;
    return out;
}
void * arena_realloc(arena_t * arena,void * ptr, size_t base_count, size_t new_count){
    if(!ptr){
        return arena_alloc(arena,new_count);
    }
    unsigned char * byte_ptr = ptr;
    if(arena->last_ptr == ptr){
        size_t offset = ((size_t)byte_ptr-(size_t)arena->bytes);
        if(offset+new_count< arena->length){
            arena->bump_offset = offset;
            return arena_alloc(arena, new_count);
        }
    }
    void *out = arena_alloc(arena, new_count);
    memcpy(out, ptr,base_count);
    return out;
}
arena_t * new_arena_sized(size_t size){
    const size_t BYTE_COUNT = size;
    arena_t * out = malloc(sizeof(arena_t) +(64-64%sizeof(arena_t))+BYTE_COUNT);
    out->bytes = (unsigned char *)out+(sizeof(arena_t) +(64-64%sizeof(arena_t)));
    out->last_ptr = 0;
    out->bump_offset = 0;
    out->length = BYTE_COUNT;
    out->next = 0;
    return out;
}

string_t arena_fmt(arena_t * arena, const char * format, ...){
    va_list args;
    va_start (args, format);
    size_t count = vsnprintf(0, 0, format, args);
    va_end(args);
    char * buffer = arena_alloc(arena, count+1);
    va_start (args, format);
    vsnprintf(buffer, count+1, format,args);
    va_end(args);
    return (string_t){.ptr = buffer, .len = count};
}

string_list_t split_string_by(arena_t * arena, string_t s, const char delimator){
    string_list_t out = new_list(arena, string_list_t);
    string_t current = (string_t){.ptr = s.ptr, .len = 0};
    for(size_t i =0; i<s.len; i++){
        if(s.ptr[i] == delimator){
            if(current.len>0){
                list_push(out, current);
            }
            current.ptr = &s.ptr[i+1];
            current.len = 0;
        }else{
            current.len += 1;
        }
    }
    if(current.len>0){
        list_push(out, current);
    }
    return out;
}

string_t read_file_to_string(arena_t * arena, char * file_name){
    FILE *f= fopen(file_name, "rb");
	if (!f){
		perror("ERROR:");
		return (string_t){
            .ptr = 0, .len =0
        };
	}
	fseek(f, 0, SEEK_END);
	size_t fsize = ftell(f);
	fseek(f, 0, SEEK_SET); 
	i8_list_t out  = new_list(arena, i8_list_t);
	list_resize(out, fsize);
	fread(out.items, 1, fsize, f);
    out.len = fsize;
	fclose(f);
	return (string_t){
        .ptr = (char*)out.items, .len = out.len
    };
}


u8_list_t read_file_to_bytes(arena_t * arena, char * file_name){
    FILE *f= fopen(file_name, "rb");
	if (!f){
		perror("ERROR:");
		return (u8_list_t){
            .items = 0, .len =0
        };
	}
	fseek(f, 0, SEEK_END);
	size_t fsize = ftell(f);
	fseek(f, 0, SEEK_SET); 
	u8_list_t out  = new_list(arena, u8_list_t);
	list_resize(out, fsize);
    out.len = fsize;
	fread(out.items, 1, fsize, f);
	fclose(f);
	return out;
}
typedef enum {
    TOKENIZATION_WHITE_SPACE;
}tokenization_state_t;
token_list_t tokenize_string(arena_t * arena,string_t file_name, string_t contents, string_list_t operators){
    token_list_t out = new_list(arena,token_list_t);
    int32_t line = 1;
    string_builder_t current = new_list(arena,string_builder_t);
    char_slice_t deliminators = SLICE(char, ',', '.', '|', ';', ':');
    char_slice_t openers = SLICE(char, '(', '[', '{');
    char_slice_t closers = SLICE(char, ')', ']', '}');
    for(size_t i =0; i<contents.len;i++){
        char c = contents.ptr[i];
        bool is_delim = false;
        bool is_opener = false;
        bool is_closer = false;
        for(size_t j =0; j<deliminators.len; j++){
            if(deliminators.items[j] == c){
                is_delim = true;
                break;
            }
        }
        for(size_t j =0; j<openers.len; j++){
            if(openers.items[j] == c){
                is_opener = true;
                break;
            }
        }
        for(size_t j =0; j<closers.len; j++){
            if(closers.items[j] == c){
                is_closer= true;
                break;
            }
        }
    }
    return out;
}

string_t string_builder_take(string_builder_t s){
    return (string_t){.ptr = s.items, .len = s.len};
}
