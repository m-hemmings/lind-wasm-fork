#include "bench.h"
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <fcntl.h>
#include <time.h>
#include <string.h>
#include <sys/wait.h>
#include <sys/socket.h>

#define LOOP_COUNT 100000

void uds_dgram(int msg_size) {
	int sv[2];
	if (socketpair(AF_UNIX, SOCK_DGRAM, 0, sv)) {
		perror("socketpair");
		exit(1);
	}

	pid_t pid = fork();

	if (pid < 0) {
		perror("fork");
		exit(1);
	}

	// Child
	if (pid == 0) {
		close(sv[0]);
		char buf[msg_size];
		for (int i = 0; i < LOOP_COUNT; i++) {
			ssize_t n = recv(sv[1], buf, msg_size, 0);
			if (n <= 0) {
				fprintf(stderr, "Received 0 bytes\n");
				exit(1);
			}
			send(sv[1], buf, n, 0);
		}
		close(sv[1]);
		_exit(0);
	}

	// Parent
	close(sv[1]);
	char buf[msg_size];
	memset(buf, 0x42, msg_size);

	long long start = gettimens();
	for (int i = 0; i < LOOP_COUNT; i++) {
		send(sv[0], buf, msg_size, 0);
		recv(sv[0], buf, msg_size, 0);
	}
	long long end = gettimens();

	emit_result("UDS DGRAM", msg_size, (end - start) / LOOP_COUNT,
		    LOOP_COUNT);
}

void uds_stream(int msg_size) {
	int sv[2];
	if (socketpair(AF_UNIX, SOCK_STREAM, 0, sv)) {
		perror("socketpair");
		exit(1);
	}

	pid_t pid = fork();

	if (pid < 0) {
		perror("fork");
		exit(1);
	}

	// Child
	if (pid == 0) {
		close(sv[0]);
		char buf[msg_size];
		for (int i = 0; i < LOOP_COUNT; i++) {
			ssize_t n = recv(sv[1], buf, msg_size, 0);
			if (n <= 0) {
				fprintf(stderr, "Received 0 bytes\n");
				exit(1);
			}
			send(sv[1], buf, n, 0);
		}
		close(sv[1]);
		_exit(0);
	}

	// Parent
	close(sv[1]);
	char buf[msg_size];
	memset(buf, 0x42, msg_size);

	long long start = gettimens();
	for (int i = 0; i < LOOP_COUNT; i++) {
		send(sv[0], buf, msg_size, 0);
		recv(sv[0], buf, msg_size, 0);
	}
	long long end = gettimens();

	emit_result("UDS STREAM", msg_size, (end - start) / LOOP_COUNT,
		    LOOP_COUNT);
}

int main() {
	for (int s = 1; s < 1024; s = s * 2) {
		uds_stream(s);
	}

	for (int s = 1; s < 1024; s = s * 2) {
		uds_dgram(s);
	}
}
