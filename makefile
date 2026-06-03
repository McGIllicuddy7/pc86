make: main.c  utils.c
	gcc main.c utils.c -std=c99 -Wall -Wextra -pedantic -fsanitize=address