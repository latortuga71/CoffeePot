#include <arpa/inet.h>
#include <errno.h>
#include <netinet/in.h>
#include <poll.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

#define MAXDATASIZE 512

void crash(char *buffer) {
  if (buffer[0] == 'A') {
    if (buffer[1] == 'B') {
      if (buffer[2] == 'C') {
        if (buffer[3] == 'D') {
          if (buffer[4] == 'E') {
            if (buffer[5] == 'F') {
              if (buffer[6] == 'G') {
                if (buffer[7] == 'H') {
                  if (buffer[8] == 'I') {
                    if (buffer[9] == 'J') {
                      if (buffer[10] == 'K') {
                        if (buffer[11] == 'L') {
                          *(int *)0 = 0;
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}

int main() {
  char *buffer = (char *)calloc(1, 512);
  int listener = socket(AF_INET, SOCK_STREAM, 0);
  struct sockaddr_in socketAddr;
  struct sockaddr clientAddr;
  socketAddr.sin_family = AF_INET;
  socketAddr.sin_port = htons(4444);
  inet_aton("0.0.0.0", &socketAddr.sin_addr);
  if (bind(listener, (struct sockaddr *)&socketAddr, sizeof(socketAddr)) ==
      -1) {
    fprintf(stderr, "Failed to bind\n");
    return -1;
  }
  if (listen(listener, 50) == -1) {
    fprintf(stderr, "Failed to listen\n");
    return -1;
  }
  socklen_t addr_size;
  int new_fd = accept(listener, (struct sockaddr *)&clientAddr, &addr_size);
  printf("Client Connected!\n");
  if (recv(new_fd, buffer, MAXDATASIZE - 1, 0) == -1) {
    fprintf(stderr, "Failed to read \n");
    return -1;
  }
  // You Want To SnapShot Here
  crash(buffer);
  // You Want To Restore Here
  printf("Got Message %s\n", buffer);
  close(new_fd);
  return 0;
}
