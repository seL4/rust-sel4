//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// https://git.musl-libc.org/cgit/musl/tree/arch/arm/bits/syscall.h.in

pub mod syscall_number {
    use crate::SyscallNumber;

    pub const RESTART_SYSCALL: SyscallNumber = 0;
    pub const EXIT: SyscallNumber = 1;
    pub const FORK: SyscallNumber = 2;
    pub const READ: SyscallNumber = 3;
    pub const WRITE: SyscallNumber = 4;
    pub const OPEN: SyscallNumber = 5;
    pub const CLOSE: SyscallNumber = 6;
    pub const CREAT: SyscallNumber = 8;
    pub const LINK: SyscallNumber = 9;
    pub const UNLINK: SyscallNumber = 10;
    pub const EXECVE: SyscallNumber = 11;
    pub const CHDIR: SyscallNumber = 12;
    pub const MKNOD: SyscallNumber = 14;
    pub const CHMOD: SyscallNumber = 15;
    pub const LCHOWN: SyscallNumber = 16;
    pub const LSEEK: SyscallNumber = 19;
    pub const GETPID: SyscallNumber = 20;
    pub const MOUNT: SyscallNumber = 21;
    pub const SETUID: SyscallNumber = 23;
    pub const GETUID: SyscallNumber = 24;
    pub const PTRACE: SyscallNumber = 26;
    pub const PAUSE: SyscallNumber = 29;
    pub const ACCESS: SyscallNumber = 33;
    pub const NICE: SyscallNumber = 34;
    pub const SYNC: SyscallNumber = 36;
    pub const KILL: SyscallNumber = 37;
    pub const RENAME: SyscallNumber = 38;
    pub const MKDIR: SyscallNumber = 39;
    pub const RMDIR: SyscallNumber = 40;
    pub const DUP: SyscallNumber = 41;
    pub const PIPE: SyscallNumber = 42;
    pub const TIMES: SyscallNumber = 43;
    pub const BRK: SyscallNumber = 45;
    pub const SETGID: SyscallNumber = 46;
    pub const GETGID: SyscallNumber = 47;
    pub const GETEUID: SyscallNumber = 49;
    pub const GETEGID: SyscallNumber = 50;
    pub const ACCT: SyscallNumber = 51;
    pub const UMOUNT2: SyscallNumber = 52;
    pub const IOCTL: SyscallNumber = 54;
    pub const FCNTL: SyscallNumber = 55;
    pub const SETPGID: SyscallNumber = 57;
    pub const UMASK: SyscallNumber = 60;
    pub const CHROOT: SyscallNumber = 61;
    pub const USTAT: SyscallNumber = 62;
    pub const DUP2: SyscallNumber = 63;
    pub const GETPPID: SyscallNumber = 64;
    pub const GETPGRP: SyscallNumber = 65;
    pub const SETSID: SyscallNumber = 66;
    pub const SIGACTION: SyscallNumber = 67;
    pub const SETREUID: SyscallNumber = 70;
    pub const SETREGID: SyscallNumber = 71;
    pub const SIGSUSPEND: SyscallNumber = 72;
    pub const SIGPENDING: SyscallNumber = 73;
    pub const SETHOSTNAME: SyscallNumber = 74;
    pub const SETRLIMIT: SyscallNumber = 75;
    pub const GETRUSAGE: SyscallNumber = 77;
    pub const GETTIMEOFDAY_TIME32: SyscallNumber = 78;
    pub const SETTIMEOFDAY_TIME32: SyscallNumber = 79;
    pub const GETGROUPS: SyscallNumber = 80;
    pub const SETGROUPS: SyscallNumber = 81;
    pub const SYMLINK: SyscallNumber = 83;
    pub const READLINK: SyscallNumber = 85;
    pub const USELIB: SyscallNumber = 86;
    pub const SWAPON: SyscallNumber = 87;
    pub const REBOOT: SyscallNumber = 88;
    pub const MUNMAP: SyscallNumber = 91;
    pub const TRUNCATE: SyscallNumber = 92;
    pub const FTRUNCATE: SyscallNumber = 93;
    pub const FCHMOD: SyscallNumber = 94;
    pub const FCHOWN: SyscallNumber = 95;
    pub const GETPRIORITY: SyscallNumber = 96;
    pub const SETPRIORITY: SyscallNumber = 97;
    pub const STATFS: SyscallNumber = 99;
    pub const FSTATFS: SyscallNumber = 100;
    pub const SYSLOG: SyscallNumber = 103;
    pub const SETITIMER: SyscallNumber = 104;
    pub const GETITIMER: SyscallNumber = 105;
    pub const STAT: SyscallNumber = 106;
    pub const LSTAT: SyscallNumber = 107;
    pub const FSTAT: SyscallNumber = 108;
    pub const VHANGUP: SyscallNumber = 111;
    pub const WAIT4: SyscallNumber = 114;
    pub const SWAPOFF: SyscallNumber = 115;
    pub const SYSINFO: SyscallNumber = 116;
    pub const FSYNC: SyscallNumber = 118;
    pub const SIGRETURN: SyscallNumber = 119;
    pub const CLONE: SyscallNumber = 120;
    pub const SETDOMAINNAME: SyscallNumber = 121;
    pub const UNAME: SyscallNumber = 122;
    pub const ADJTIMEX: SyscallNumber = 124;
    pub const MPROTECT: SyscallNumber = 125;
    pub const SIGPROCMASK: SyscallNumber = 126;
    pub const INIT_MODULE: SyscallNumber = 128;
    pub const DELETE_MODULE: SyscallNumber = 129;
    pub const QUOTACTL: SyscallNumber = 131;
    pub const GETPGID: SyscallNumber = 132;
    pub const FCHDIR: SyscallNumber = 133;
    pub const BDFLUSH: SyscallNumber = 134;
    pub const SYSFS: SyscallNumber = 135;
    pub const PERSONALITY: SyscallNumber = 136;
    pub const SETFSUID: SyscallNumber = 138;
    pub const SETFSGID: SyscallNumber = 139;
    pub const _LLSEEK: SyscallNumber = 140;
    pub const GETDENTS: SyscallNumber = 141;
    pub const _NEWSELECT: SyscallNumber = 142;
    pub const FLOCK: SyscallNumber = 143;
    pub const MSYNC: SyscallNumber = 144;
    pub const READV: SyscallNumber = 145;
    pub const WRITEV: SyscallNumber = 146;
    pub const GETSID: SyscallNumber = 147;
    pub const FDATASYNC: SyscallNumber = 148;
    pub const _SYSCTL: SyscallNumber = 149;
    pub const MLOCK: SyscallNumber = 150;
    pub const MUNLOCK: SyscallNumber = 151;
    pub const MLOCKALL: SyscallNumber = 152;
    pub const MUNLOCKALL: SyscallNumber = 153;
    pub const SCHED_SETPARAM: SyscallNumber = 154;
    pub const SCHED_GETPARAM: SyscallNumber = 155;
    pub const SCHED_SETSCHEDULER: SyscallNumber = 156;
    pub const SCHED_GETSCHEDULER: SyscallNumber = 157;
    pub const SCHED_YIELD: SyscallNumber = 158;
    pub const SCHED_GET_PRIORITY_MAX: SyscallNumber = 159;
    pub const SCHED_GET_PRIORITY_MIN: SyscallNumber = 160;
    pub const SCHED_RR_GET_INTERVAL: SyscallNumber = 161;
    pub const NANOSLEEP: SyscallNumber = 162;
    pub const MREMAP: SyscallNumber = 163;
    pub const SETRESUID: SyscallNumber = 164;
    pub const GETRESUID: SyscallNumber = 165;
    pub const POLL: SyscallNumber = 168;
    pub const NFSSERVCTL: SyscallNumber = 169;
    pub const SETRESGID: SyscallNumber = 170;
    pub const GETRESGID: SyscallNumber = 171;
    pub const PRCTL: SyscallNumber = 172;
    pub const RT_SIGRETURN: SyscallNumber = 173;
    pub const RT_SIGACTION: SyscallNumber = 174;
    pub const RT_SIGPROCMASK: SyscallNumber = 175;
    pub const RT_SIGPENDING: SyscallNumber = 176;
    pub const RT_SIGTIMEDWAIT: SyscallNumber = 177;
    pub const RT_SIGQUEUEINFO: SyscallNumber = 178;
    pub const RT_SIGSUSPEND: SyscallNumber = 179;
    pub const PREAD64: SyscallNumber = 180;
    pub const PWRITE64: SyscallNumber = 181;
    pub const CHOWN: SyscallNumber = 182;
    pub const GETCWD: SyscallNumber = 183;
    pub const CAPGET: SyscallNumber = 184;
    pub const CAPSET: SyscallNumber = 185;
    pub const SIGALTSTACK: SyscallNumber = 186;
    pub const SENDFILE: SyscallNumber = 187;
    pub const VFORK: SyscallNumber = 190;
    pub const UGETRLIMIT: SyscallNumber = 191;
    pub const MMAP2: SyscallNumber = 192;
    pub const TRUNCATE64: SyscallNumber = 193;
    pub const FTRUNCATE64: SyscallNumber = 194;
    pub const STAT64: SyscallNumber = 195;
    pub const LSTAT64: SyscallNumber = 196;
    pub const FSTAT64: SyscallNumber = 197;
    pub const LCHOWN32: SyscallNumber = 198;
    pub const GETUID32: SyscallNumber = 199;
    pub const GETGID32: SyscallNumber = 200;
    pub const GETEUID32: SyscallNumber = 201;
    pub const GETEGID32: SyscallNumber = 202;
    pub const SETREUID32: SyscallNumber = 203;
    pub const SETREGID32: SyscallNumber = 204;
    pub const GETGROUPS32: SyscallNumber = 205;
    pub const SETGROUPS32: SyscallNumber = 206;
    pub const FCHOWN32: SyscallNumber = 207;
    pub const SETRESUID32: SyscallNumber = 208;
    pub const GETRESUID32: SyscallNumber = 209;
    pub const SETRESGID32: SyscallNumber = 210;
    pub const GETRESGID32: SyscallNumber = 211;
    pub const CHOWN32: SyscallNumber = 212;
    pub const SETUID32: SyscallNumber = 213;
    pub const SETGID32: SyscallNumber = 214;
    pub const SETFSUID32: SyscallNumber = 215;
    pub const SETFSGID32: SyscallNumber = 216;
    pub const GETDENTS64: SyscallNumber = 217;
    pub const PIVOT_ROOT: SyscallNumber = 218;
    pub const MINCORE: SyscallNumber = 219;
    pub const MADVISE: SyscallNumber = 220;
    pub const FCNTL64: SyscallNumber = 221;
    pub const GETTID: SyscallNumber = 224;
    pub const READAHEAD: SyscallNumber = 225;
    pub const SETXATTR: SyscallNumber = 226;
    pub const LSETXATTR: SyscallNumber = 227;
    pub const FSETXATTR: SyscallNumber = 228;
    pub const GETXATTR: SyscallNumber = 229;
    pub const LGETXATTR: SyscallNumber = 230;
    pub const FGETXATTR: SyscallNumber = 231;
    pub const LISTXATTR: SyscallNumber = 232;
    pub const LLISTXATTR: SyscallNumber = 233;
    pub const FLISTXATTR: SyscallNumber = 234;
    pub const REMOVEXATTR: SyscallNumber = 235;
    pub const LREMOVEXATTR: SyscallNumber = 236;
    pub const FREMOVEXATTR: SyscallNumber = 237;
    pub const TKILL: SyscallNumber = 238;
    pub const SENDFILE64: SyscallNumber = 239;
    pub const FUTEX: SyscallNumber = 240;
    pub const SCHED_SETAFFINITY: SyscallNumber = 241;
    pub const SCHED_GETAFFINITY: SyscallNumber = 242;
    pub const IO_SETUP: SyscallNumber = 243;
    pub const IO_DESTROY: SyscallNumber = 244;
    pub const IO_GETEVENTS: SyscallNumber = 245;
    pub const IO_SUBMIT: SyscallNumber = 246;
    pub const IO_CANCEL: SyscallNumber = 247;
    pub const EXIT_GROUP: SyscallNumber = 248;
    pub const LOOKUP_DCOOKIE: SyscallNumber = 249;
    pub const EPOLL_CREATE: SyscallNumber = 250;
    pub const EPOLL_CTL: SyscallNumber = 251;
    pub const EPOLL_WAIT: SyscallNumber = 252;
    pub const REMAP_FILE_PAGES: SyscallNumber = 253;
    pub const SET_TID_ADDRESS: SyscallNumber = 256;
    pub const TIMER_CREATE: SyscallNumber = 257;
    pub const TIMER_SETTIME32: SyscallNumber = 258;
    pub const TIMER_GETTIME32: SyscallNumber = 259;
    pub const TIMER_GETOVERRUN: SyscallNumber = 260;
    pub const TIMER_DELETE: SyscallNumber = 261;
    pub const CLOCK_SETTIME32: SyscallNumber = 262;
    pub const CLOCK_GETTIME32: SyscallNumber = 263;
    pub const CLOCK_GETRES_TIME32: SyscallNumber = 264;
    pub const CLOCK_NANOSLEEP_TIME32: SyscallNumber = 265;
    pub const STATFS64: SyscallNumber = 266;
    pub const FSTATFS64: SyscallNumber = 267;
    pub const TGKILL: SyscallNumber = 268;
    pub const UTIMES: SyscallNumber = 269;
    pub const FADVISE64_64: SyscallNumber = 270;
    pub const ARM_FADVISE64_64: SyscallNumber = 270;
    pub const PCICONFIG_IOBASE: SyscallNumber = 271;
    pub const PCICONFIG_READ: SyscallNumber = 272;
    pub const PCICONFIG_WRITE: SyscallNumber = 273;
    pub const MQ_OPEN: SyscallNumber = 274;
    pub const MQ_UNLINK: SyscallNumber = 275;
    pub const MQ_TIMEDSEND: SyscallNumber = 276;
    pub const MQ_TIMEDRECEIVE: SyscallNumber = 277;
    pub const MQ_NOTIFY: SyscallNumber = 278;
    pub const MQ_GETSETATTR: SyscallNumber = 279;
    pub const WAITID: SyscallNumber = 280;
    pub const SOCKET: SyscallNumber = 281;
    pub const BIND: SyscallNumber = 282;
    pub const CONNECT: SyscallNumber = 283;
    pub const LISTEN: SyscallNumber = 284;
    pub const ACCEPT: SyscallNumber = 285;
    pub const GETSOCKNAME: SyscallNumber = 286;
    pub const GETPEERNAME: SyscallNumber = 287;
    pub const SOCKETPAIR: SyscallNumber = 288;
    pub const SEND: SyscallNumber = 289;
    pub const SENDTO: SyscallNumber = 290;
    pub const RECV: SyscallNumber = 291;
    pub const RECVFROM: SyscallNumber = 292;
    pub const SHUTDOWN: SyscallNumber = 293;
    pub const SETSOCKOPT: SyscallNumber = 294;
    pub const GETSOCKOPT: SyscallNumber = 295;
    pub const SENDMSG: SyscallNumber = 296;
    pub const RECVMSG: SyscallNumber = 297;
    pub const SEMOP: SyscallNumber = 298;
    pub const SEMGET: SyscallNumber = 299;
    pub const SEMCTL: SyscallNumber = 300;
    pub const MSGSND: SyscallNumber = 301;
    pub const MSGRCV: SyscallNumber = 302;
    pub const MSGGET: SyscallNumber = 303;
    pub const MSGCTL: SyscallNumber = 304;
    pub const SHMAT: SyscallNumber = 305;
    pub const SHMDT: SyscallNumber = 306;
    pub const SHMGET: SyscallNumber = 307;
    pub const SHMCTL: SyscallNumber = 308;
    pub const ADD_KEY: SyscallNumber = 309;
    pub const REQUEST_KEY: SyscallNumber = 310;
    pub const KEYCTL: SyscallNumber = 311;
    pub const SEMTIMEDOP: SyscallNumber = 312;
    pub const VSERVER: SyscallNumber = 313;
    pub const IOPRIO_SET: SyscallNumber = 314;
    pub const IOPRIO_GET: SyscallNumber = 315;
    pub const INOTIFY_INIT: SyscallNumber = 316;
    pub const INOTIFY_ADD_WATCH: SyscallNumber = 317;
    pub const INOTIFY_RM_WATCH: SyscallNumber = 318;
    pub const MBIND: SyscallNumber = 319;
    pub const GET_MEMPOLICY: SyscallNumber = 320;
    pub const SET_MEMPOLICY: SyscallNumber = 321;
    pub const OPENAT: SyscallNumber = 322;
    pub const MKDIRAT: SyscallNumber = 323;
    pub const MKNODAT: SyscallNumber = 324;
    pub const FCHOWNAT: SyscallNumber = 325;
    pub const FUTIMESAT: SyscallNumber = 326;
    pub const FSTATAT64: SyscallNumber = 327;
    pub const UNLINKAT: SyscallNumber = 328;
    pub const RENAMEAT: SyscallNumber = 329;
    pub const LINKAT: SyscallNumber = 330;
    pub const SYMLINKAT: SyscallNumber = 331;
    pub const READLINKAT: SyscallNumber = 332;
    pub const FCHMODAT: SyscallNumber = 333;
    pub const FACCESSAT: SyscallNumber = 334;
    pub const PSELECT6: SyscallNumber = 335;
    pub const PPOLL: SyscallNumber = 336;
    pub const UNSHARE: SyscallNumber = 337;
    pub const SET_ROBUST_LIST: SyscallNumber = 338;
    pub const GET_ROBUST_LIST: SyscallNumber = 339;
    pub const SPLICE: SyscallNumber = 340;
    pub const SYNC_FILE_RANGE2: SyscallNumber = 341;
    pub const ARM_SYNC_FILE_RANGE: SyscallNumber = 341;
    pub const TEE: SyscallNumber = 342;
    pub const VMSPLICE: SyscallNumber = 343;
    pub const MOVE_PAGES: SyscallNumber = 344;
    pub const GETCPU: SyscallNumber = 345;
    pub const EPOLL_PWAIT: SyscallNumber = 346;
    pub const KEXEC_LOAD: SyscallNumber = 347;
    pub const UTIMENSAT: SyscallNumber = 348;
    pub const SIGNALFD: SyscallNumber = 349;
    pub const TIMERFD_CREATE: SyscallNumber = 350;
    pub const EVENTFD: SyscallNumber = 351;
    pub const FALLOCATE: SyscallNumber = 352;
    pub const TIMERFD_SETTIME32: SyscallNumber = 353;
    pub const TIMERFD_GETTIME32: SyscallNumber = 354;
    pub const SIGNALFD4: SyscallNumber = 355;
    pub const EVENTFD2: SyscallNumber = 356;
    pub const EPOLL_CREATE1: SyscallNumber = 357;
    pub const DUP3: SyscallNumber = 358;
    pub const PIPE2: SyscallNumber = 359;
    pub const INOTIFY_INIT1: SyscallNumber = 360;
    pub const PREADV: SyscallNumber = 361;
    pub const PWRITEV: SyscallNumber = 362;
    pub const RT_TGSIGQUEUEINFO: SyscallNumber = 363;
    pub const PERF_EVENT_OPEN: SyscallNumber = 364;
    pub const RECVMMSG: SyscallNumber = 365;
    pub const ACCEPT4: SyscallNumber = 366;
    pub const FANOTIFY_INIT: SyscallNumber = 367;
    pub const FANOTIFY_MARK: SyscallNumber = 368;
    pub const PRLIMIT64: SyscallNumber = 369;
    pub const NAME_TO_HANDLE_AT: SyscallNumber = 370;
    pub const OPEN_BY_HANDLE_AT: SyscallNumber = 371;
    pub const CLOCK_ADJTIME: SyscallNumber = 372;
    pub const SYNCFS: SyscallNumber = 373;
    pub const SENDMMSG: SyscallNumber = 374;
    pub const SETNS: SyscallNumber = 375;
    pub const PROCESS_VM_READV: SyscallNumber = 376;
    pub const PROCESS_VM_WRITEV: SyscallNumber = 377;
    pub const KCMP: SyscallNumber = 378;
    pub const FINIT_MODULE: SyscallNumber = 379;
    pub const SCHED_SETATTR: SyscallNumber = 380;
    pub const SCHED_GETATTR: SyscallNumber = 381;
    pub const RENAMEAT2: SyscallNumber = 382;
    pub const SECCOMP: SyscallNumber = 383;
    pub const GETRANDOM: SyscallNumber = 384;
    pub const MEMFD_CREATE: SyscallNumber = 385;
    pub const BPF: SyscallNumber = 386;
    pub const EXECVEAT: SyscallNumber = 387;
    pub const USERFAULTFD: SyscallNumber = 388;
    pub const MEMBARRIER: SyscallNumber = 389;
    pub const MLOCK2: SyscallNumber = 390;
    pub const COPY_FILE_RANGE: SyscallNumber = 391;
    pub const PREADV2: SyscallNumber = 392;
    pub const PWRITEV2: SyscallNumber = 393;
    pub const PKEY_MPROTECT: SyscallNumber = 394;
    pub const PKEY_ALLOC: SyscallNumber = 395;
    pub const PKEY_FREE: SyscallNumber = 396;
    pub const STATX: SyscallNumber = 397;
    pub const RSEQ: SyscallNumber = 398;
    pub const IO_PGETEVENTS: SyscallNumber = 399;
    pub const MIGRATE_PAGES: SyscallNumber = 400;
    pub const KEXEC_FILE_LOAD: SyscallNumber = 401;
    pub const CLOCK_GETTIME64: SyscallNumber = 403;
    pub const CLOCK_SETTIME64: SyscallNumber = 404;
    pub const CLOCK_ADJTIME64: SyscallNumber = 405;
    pub const CLOCK_GETRES_TIME64: SyscallNumber = 406;
    pub const CLOCK_NANOSLEEP_TIME64: SyscallNumber = 407;
    pub const TIMER_GETTIME64: SyscallNumber = 408;
    pub const TIMER_SETTIME64: SyscallNumber = 409;
    pub const TIMERFD_GETTIME64: SyscallNumber = 410;
    pub const TIMERFD_SETTIME64: SyscallNumber = 411;
    pub const UTIMENSAT_TIME64: SyscallNumber = 412;
    pub const PSELECT6_TIME64: SyscallNumber = 413;
    pub const PPOLL_TIME64: SyscallNumber = 414;
    pub const IO_PGETEVENTS_TIME64: SyscallNumber = 416;
    pub const RECVMMSG_TIME64: SyscallNumber = 417;
    pub const MQ_TIMEDSEND_TIME64: SyscallNumber = 418;
    pub const MQ_TIMEDRECEIVE_TIME64: SyscallNumber = 419;
    pub const SEMTIMEDOP_TIME64: SyscallNumber = 420;
    pub const RT_SIGTIMEDWAIT_TIME64: SyscallNumber = 421;
    pub const FUTEX_TIME64: SyscallNumber = 422;
    pub const SCHED_RR_GET_INTERVAL_TIME64: SyscallNumber = 423;
    pub const PIDFD_SEND_SIGNAL: SyscallNumber = 424;
    pub const IO_URING_SETUP: SyscallNumber = 425;
    pub const IO_URING_ENTER: SyscallNumber = 426;
    pub const IO_URING_REGISTER: SyscallNumber = 427;
    pub const OPEN_TREE: SyscallNumber = 428;
    pub const MOVE_MOUNT: SyscallNumber = 429;
    pub const FSOPEN: SyscallNumber = 430;
    pub const FSCONFIG: SyscallNumber = 431;
    pub const FSMOUNT: SyscallNumber = 432;
    pub const FSPICK: SyscallNumber = 433;
    pub const PIDFD_OPEN: SyscallNumber = 434;
    pub const CLONE3: SyscallNumber = 435;
    pub const CLOSE_RANGE: SyscallNumber = 436;
    pub const OPENAT2: SyscallNumber = 437;
    pub const PIDFD_GETFD: SyscallNumber = 438;
    pub const FACCESSAT2: SyscallNumber = 439;
    pub const PROCESS_MADVISE: SyscallNumber = 440;
    pub const EPOLL_PWAIT2: SyscallNumber = 441;
    pub const MOUNT_SETATTR: SyscallNumber = 442;
    pub const LANDLOCK_CREATE_RULESET: SyscallNumber = 444;
    pub const LANDLOCK_ADD_RULE: SyscallNumber = 445;
    pub const LANDLOCK_RESTRICT_SELF: SyscallNumber = 446;
    pub const PROCESS_MRELEASE: SyscallNumber = 448;
    pub const FUTEX_WAITV: SyscallNumber = 449;
    pub const SET_MEMPOLICY_HOME_NODE: SyscallNumber = 450;
    pub const CACHESTAT: SyscallNumber = 451;
    pub const FCHMODAT2: SyscallNumber = 452;
}

pub mod arm_syscall_number {
    use crate::SyscallNumber;

    pub const BREAKPOINT: SyscallNumber = 0x0f0001;
    pub const CACHEFLUSH: SyscallNumber = 0x0f0002;
    pub const USR26: SyscallNumber = 0x0f0003;
    pub const USR32: SyscallNumber = 0x0f0004;
    pub const SET_TLS: SyscallNumber = 0x0f0005;
    pub const GET_TLS: SyscallNumber = 0x0f0006;
}
