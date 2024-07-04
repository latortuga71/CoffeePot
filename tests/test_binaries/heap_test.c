#include <stdlib.h>



int main(int argc, char** argv){
	void* buffer1 = malloc(1);
	void* buffer2 = malloc(512);
	void* buffer3 = malloc(0x1024);
	void* buffer4 = malloc(0x1024*0x1024);
	free(buffer1);
	free(buffer2);
	free(buffer3);
	free(buffer4);
	return 0;
}
