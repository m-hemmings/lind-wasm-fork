// Useful for defining IPC and FS test message sizes.
#define KiB(x) ((size_t)(x) << 10)
#define MiB(x) ((size_t)(x) << 20)

long long gettimens();

// Prints out the benchmark result in a benchrunner.py friendy format.
void emit_result(char*, int, long long, int);
