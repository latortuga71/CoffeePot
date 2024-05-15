build:
	clang++ ./src/* -o ./bin/coffeepot -I ./include/
	clang++ ./src/emulator.cc ./src/loader.cc ./tests/tests.c -o ./tests/coffeepot_tests -I ./include/

test:
	./tests/coffeepot_tests
run:
	./bin/coffeepot ./tests/hello_test.bin
