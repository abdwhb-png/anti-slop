#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <signal.h>
#include <stdio.h>
#include <unistd.h>
int kill(pid_t pid, int sig) {
    static int (*real_kill)(pid_t, int);
    if (!real_kill) real_kill = dlsym(RTLD_NEXT, "kill");
    int saved = errno;
    if (pid < 0 && sig == SIGKILL) {
        char path[64];
        snprintf(path, sizeof(path), "/proc/%ld", -(long)pid);
        if (access(path, F_OK) == -1 && errno == ENOENT) {
            char message[128];
            int size = snprintf(message, sizeof(message), "OBSERVED_GROUP_SIGNAL_AFTER_LEADER_REAP pid=%ld signal=%d\n", -(long)pid, sig);
            (void)write(STDERR_FILENO, message, size);
        }
    }
    errno = saved;
    return real_kill(pid, sig);
}
