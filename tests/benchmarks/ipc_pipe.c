#include "bench.h"
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <fcntl.h>
#include <time.h>
#include <string.h>
#include <sys/wait.h>

#define LOOP_COUNT 100000

void bench_pipe(int msg_size) {
	int p2c[2], c2p[2];

	if (pipe(p2c) || pipe(c2p)) {
		perror("pipe");
		exit(1);
	}

	pid_t pid = fork();
	if (pid < 0) {
		perror("fork");
		exit(1);
	}

	// Child
	if (pid == 0) {
		close(p2c[1]);
		close(c2p[0]);

		char buf[msg_size];
		for (int i = 0; i < LOOP_COUNT; i++) {
			ssize_t n = read(p2c[0], buf, msg_size);
			if (n <= 0) {
				fprintf(stderr, "0 bytes read\n");
				exit(1);
			}
			write(c2p[1], buf, n);
		}

		close(p2c[0]);
		close(c2p[0]);
		_exit(0);
	}

	// Parent
	close(p2c[0]);
	close(c2p[1]);
	char buf[msg_size];
	memset(buf, 0x42, msg_size);

	long long t0 = gettimens();
	for (int i = 0; i < LOOP_COUNT; i++) {
		write(p2c[1], buf, msg_size);
		read(c2p[0], buf, msg_size);
	}
	long long t1 = gettimens();

	close(p2c[1]);
	close(c2p[0]);
	wait(NULL);

	emit_result("PIPE", msg_size, (t1 - t0) / LOOP_COUNT, LOOP_COUNT);
}

int main() {
	for (int s = 1; s < 1024; s = s * 2) {
		bench_pipe(s);
	}
}
