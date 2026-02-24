#include "bench.h"
#include <time.h>
#include <stdio.h>

long long gettimens() {
	struct timespec tp;
	clock_gettime(CLOCK_MONOTONIC, &tp);
	return (long long)tp.tv_sec * 1000000000LL + tp.tv_nsec;
}

inline void emit_result(char *test, int param, long long average, int loops) {
	printf("%s\t%d\t%d\n%lld\n", test, param, loops, average);
}
