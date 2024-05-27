build:
	clang++ ./src/* -o ./bin/coffeepot -I ./include/ -w

test:
	clang++ ./src/emulator.cc ./src/loader.cc ./tests/tests.c -o ./tests/coffeepot_tests -I ./include/ -w && ./tests/coffeepot_tests
run:
	./bin/coffeepot ./tests/hello_test.bin
