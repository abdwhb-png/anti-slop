#include <stddef.h>
#include <signal.h>
#include <stdio.h>
#include <sys/wait.h>
int main(void) {
    printf("siginfo_t size=%zu align=%zu si_signo_offset=%zu P_PID=%d WEXITED=%d WNOHANG=%d WNOWAIT=%#x\n", sizeof(siginfo_t), _Alignof(siginfo_t), offsetof(siginfo_t, si_signo), P_PID, WEXITED, WNOHANG, WNOWAIT);
    return !(sizeof(siginfo_t) == 128 && _Alignof(siginfo_t) <= 8 && offsetof(siginfo_t, si_signo) == 0 && P_PID == 1 && WEXITED == 4 && WNOHANG == 1 && WNOWAIT == 0x01000000);
}
