#include "bench.h"
#include "imfs.h"

#include <fcntl.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdio.h>

#define LOOP_COUNT(size) ((size) > 4096 ? 1000 : 1000000)

void read_size(size_t count) {
	char *buf = malloc(count); // [MAX];

	int fd = imfs_open(0, "tmp_fs_read.txt", O_RDONLY, 0);

	int loops = LOOP_COUNT(count);

	long long start_time = gettimens();
	for (int i = 0; i < loops; i++) {
		imfs_pread(0, fd, buf, count, 0);
	}
	long long end_time = gettimens();

	long long avg_time = (end_time - start_time) / loops;
	emit_result("IMFS Read", count, avg_time, loops);

	imfs_close(0, fd);
}

int main(int argc, char *argv[]) {
	imfs_init();

	int sizes[4] = {1, KiB(1), KiB(4), KiB(10)}; // MiB(1), MiB(10)};

	char wchar = 'A';

	int fd = imfs_open(0, "tmp_fs_read.txt", O_CREAT | O_WRONLY, 0666);
	for (int i = 0; i < KiB(10); i++) {
		imfs_write(0, fd, &wchar, 1);
	}
	close(fd);

	// Run benchmarks.
	for (int i = 0; i < 4; i++) {
		read_size(sizes[i]);
	}

	unlink("tmp_fs_read.txt");
}
