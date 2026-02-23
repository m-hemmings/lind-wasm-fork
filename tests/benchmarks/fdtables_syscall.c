/*
 * Benchmark for a syscall that goes to a RawPOSIX handler that
 * does not go to the kernel, instead gets resolved within fdtables.
 *
 * Implemented as an alias of close(-1);
 *
 * Run with `sudo lind-boot --perf fdtcall.wasm`
 */

#include <lind_syscall.h>
#include <stdio.h>

#define LOOP_COUNT	1000000

int main() {
	int ret;
	for (int i=0; i < LOOP_COUNT; i++) {
		ret = fdt_syscall();
	}

	printf(".");
}
