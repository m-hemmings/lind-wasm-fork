/*
Write failure scenarios for path conversion exploits
*/

#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>
#include <sys/stat.h>
#include <string.h>
#include <assert.h>

int main() {
    int fd;
    int ret;

    /* Test 1: open(NULL) - should return -1, not crash */
    errno = 0;
    fd = open(NULL, O_RDONLY);
    assert(fd == -1 && "open(NULL) should return -1");
    printf("Test 1 PASS: open(NULL) returned -1\n");

    /* Test 2: stat(NULL) should return -1, not crash */
    errno = 0;
    struct stat st;
    ret = stat(NULL, &st);
    assert(ret == -1 && "stat(NULL) should return -1");
    printf("Test 2 PASS: stat(NULL) returned -1\n");

    /* Test 3: access(NULL) should return -1, not crash */
    errno = 0;
    ret = access(NULL, F_OK);
    assert(ret == -1 && "access(NULL) should return -1");
    printf("Test 3 PASS: access(NULL) returned -1\n");

    /* Test 4: mkdir(NULL, ...) — should return -1, not crash */
    errno = 0;
    ret = mkdir(NULL, 0755);
    assert(ret == -1 && "mkdir(NULL) should return -1");
    printf("Test 4 PASS: mkdir(NULL) returned -1\n");

    /* Test 5: unlink(NULL) — should return -1, not crash */
    errno = 0;
    ret = unlink(NULL);
    assert(ret == -1 && "unlink(NULL) should return -1");
    printf("Test 5 PASS: unlink(NULL) returned -1\n");

    /* Test 6: link(NULL, NULL) — should return -1, not crash */
    errno = 0;
    ret = link(NULL, NULL);
    assert(ret == -1 && "link(NULL, NULL) should return -1");
    printf("Test 6 PASS: link(NULL, NULL) returned -1\n");

    /* Test 7: rename(NULL, NULL) — should return -1, not crash */
    errno = 0;
    ret = rename(NULL, NULL);
    assert(ret == -1 && "rename(NULL, NULL) should return -1");
    printf("Test 7 PASS: rename(NULL, NULL) returned -1\n");

    printf("All path_conversion safety tests completed without crash.\n");
    fflush(stdout);
    return 0;
}
