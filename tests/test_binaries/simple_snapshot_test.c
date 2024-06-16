#include <stdio.h>

int main(){
	int panic = 0;
	char* buffer = "Hello from coffeepot!\n";
	if (buffer[0] == 'A')
		if (buffer[1] == 'B')
			if (buffer[2] == 'C')
				if (buffer[3] == 'D')
					panic = *(int*)0x0;
	printf(buffer);
	return 0;
}
