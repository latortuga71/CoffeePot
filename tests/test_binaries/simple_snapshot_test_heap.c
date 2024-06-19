#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(){
	int panic = 0;
	char* buffer = (char*)malloc(512);
	strcpy(buffer,"Hello from coffeepot!\n");
	if (buffer[0] == 'A')
		if (buffer[1] == 'B')
			if (buffer[2] == 'C')
				if (buffer[3] == 'D')
					if (buffer[4] == 'E')
						if (buffer[5] == 'F')
							if (buffer[6] == 'G')
								if (buffer[7] == 'H')
									if (buffer[8] == 'I')
										if (buffer[9] == 'J')
											if (buffer[10] == 'K')
												*((char *)NULL) = 0;


	printf(buffer);
	return 0;
}
