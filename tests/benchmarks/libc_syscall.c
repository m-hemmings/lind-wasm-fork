/*
 * Benchmark for a syscall that goes to a RawPOSIX handler that
 * goes to the Linux kernel with minimal processing.
 *
 * Implemented as an alias of geteuid();
 *
 * Run with `sudo lind-boot --perf libccall.wasm`
 */

#include <lind_syscall.h>
#include <stdio.h>

#define LOOP_COUNT	1000000

int main() {
	int ret;
	for (int i=0; i < LOOP_COUNT; i++) {
		ret = libc_syscall();
	}

	printf(".");
}
