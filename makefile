make: main.c 
	clang main.c -c -fPIC -undefined dynamic_lookup
	clang main.o -shared -o main.so -fPIC -undefined dynamic_lookup
	rm main.o