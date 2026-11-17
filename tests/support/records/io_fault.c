#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdbool.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

/*
 * Test-only Linux interposer for deterministic SQLite WAL commit faults.
 * Modes: write  -> fail WAL pwrite/write after the enable file exists
 *        over   -> after a WAL fdatasync, create a large extra file
 *        unmeasure -> after a WAL fdatasync, rename the database main file
 */

static const char *mode(void) {
    const char *value = getenv("AS_IO_FAULT");
    return value ? value : "";
}

static bool enabled(void) {
    const char *path = getenv("AS_FAIL_ENABLE");
    return path && access(path, F_OK) == 0;
}

static bool is_wal(int fd) {
    char link[64];
    char path[PATH_MAX];
    snprintf(link, sizeof link, "/proc/self/fd/%d", fd);
    ssize_t length = readlink(link, path, sizeof path - 1);
    if (length <= 0) return false;
    path[length] = 0;
    const char *database = getenv("AS_FAULT_DB");
    if (database) {
        size_t size = strlen(database);
        return strncmp(path, database, size) == 0 && strcmp(path + size, "-wal") == 0;
    }
    size_t size = strlen(path);
    return size >= 4 && strcmp(path + size - 4, "-wal") == 0;
}

static bool fired_once(void) {
    static bool fired = false;
    bool was = fired;
    fired = true;
    return was;
}

static void fault_after_sync(void) {
    if (!enabled() || fired_once()) return;
    if (strcmp(mode(), "over") == 0) {
        const char *extra = getenv("AS_FAULT_EXTRA");
        if (!extra) return;
        int (*real_open)(const char *, int, ...) = dlsym(RTLD_NEXT, "open");
        ssize_t (*real_write)(int, const void *, size_t) = dlsym(RTLD_NEXT, "write");
        int (*real_close)(int) = dlsym(RTLD_NEXT, "close");
        if (!real_open || !real_write || !real_close) return;
        int fd = real_open(extra, O_CREAT | O_WRONLY | O_TRUNC, 0600);
        if (fd < 0) return;
        char block[4096];
        memset(block, 'j', sizeof block);
        for (int index = 0; index < 512; index++) {
            if (real_write(fd, block, sizeof block) != (ssize_t)sizeof block) break;
        }
        real_close(fd);
    } else if (strcmp(mode(), "unmeasure") == 0) {
        const char *database = getenv("AS_FAULT_DB");
        if (!database) return;
        char moved[PATH_MAX];
        snprintf(moved, sizeof moved, "%s.moved", database);
        int (*real_rename)(const char *, const char *) = dlsym(RTLD_NEXT, "rename");
        if (real_rename) real_rename(database, moved);
    }
}

static void pause_before_sync(void) {
    if (strcmp(mode(), "pause") != 0 || !enabled() || fired_once()) return;
    const char *ready = getenv("AS_PAUSE_READY");
    if (ready) {
        int (*real_open)(const char *, int, ...) = dlsym(RTLD_NEXT, "open");
        ssize_t (*real_write)(int, const void *, size_t) = dlsym(RTLD_NEXT, "write");
        int (*real_close)(int) = dlsym(RTLD_NEXT, "close");
        if (real_open && real_write && real_close) {
            int fd = real_open(ready, O_CREAT | O_WRONLY | O_TRUNC, 0600);
            if (fd >= 0) {
                real_write(fd, "ready\n", 6);
                real_close(fd);
            }
        }
    }
    raise(SIGSTOP);
}

static bool fail_wal_write(int fd) {
    if (strcmp(mode(), "write") != 0 || !enabled() || !is_wal(fd)) return false;
    errno = EIO;
    return true;
}

ssize_t write(int fd, const void *buffer, size_t count) {
    static ssize_t (*real_write)(int, const void *, size_t);
    if (!real_write) real_write = dlsym(RTLD_NEXT, "write");
    if (fail_wal_write(fd)) return -1;
    return real_write(fd, buffer, count);
}

ssize_t pwrite(int fd, const void *buffer, size_t count, off_t offset) {
    static ssize_t (*real_pwrite)(int, const void *, size_t, off_t);
    if (!real_pwrite) real_pwrite = dlsym(RTLD_NEXT, "pwrite");
    if (fail_wal_write(fd)) return -1;
    return real_pwrite(fd, buffer, count, offset);
}

ssize_t pwrite64(int fd, const void *buffer, size_t count, off64_t offset) {
    static ssize_t (*real_pwrite64)(int, const void *, size_t, off64_t);
    if (!real_pwrite64) real_pwrite64 = dlsym(RTLD_NEXT, "pwrite64");
    if (fail_wal_write(fd)) return -1;
    return real_pwrite64(fd, buffer, count, offset);
}

int fsync(int fd) {
    static int (*real_fsync)(int);
    if (!real_fsync) real_fsync = dlsym(RTLD_NEXT, "fsync");
    if (is_wal(fd)) pause_before_sync();
    int result = real_fsync(fd);
    if (result == 0 && is_wal(fd)) fault_after_sync();
    return result;
}

int fdatasync(int fd) {
    static int (*real_fdatasync)(int);
    if (!real_fdatasync) real_fdatasync = dlsym(RTLD_NEXT, "fdatasync");
    if (is_wal(fd)) pause_before_sync();
    int result = real_fdatasync(fd);
    if (result == 0 && is_wal(fd)) fault_after_sync();
    return result;
}
