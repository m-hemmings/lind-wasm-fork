#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <assert.h>

int main(int argc, char *argv[]) {
    int ret_euid = geteuid();
    if (ret_euid != 10) {
        fprintf(stderr, "[Cage | multi-register] FAIL: geteuid expected 10, got %d\n", ret_euid);
        assert(0);
        exit(EXIT_FAILURE);
    }
    printf("[Cage | multi-register] PASS: geteuid=%d, getuid=%d\n", ret_euid, ret_uid);
    return 0;
}
