## lab5实验报告

#### 实验结果

Usertests: Test ch8b_mpsc_sem in Process 48 exited with code 0
Usertests: Test ch8b_phil_din_mutex in Process 59 exited with code 0
Usertests: Test ch8b_race_adder_mutex_spin in Process 61 exited with code 0
Usertests: Test ch8b_sync_sem in Process 62 exited with code 0
Usertests: Test ch8b_test_condvar in Process 63 exited with code 0
Usertests: Test ch8b_threads in Process 64 exited with code 0
Usertests: Test ch8b_threads_arg in Process 65 exited with code 0
ch8 Usertests passed!

Usertests: Test ch5b_forktest2 in Process 13 exited with code 0
Usertests: Test ch6b_filetest_simple in Process 14 exited with code 0
Usertests: Test ch7b_pipetest in Process 15 exited with code 0
Usertests: Test ch8_deadlock_mutex1 in Process 16 exited with code 0
Usertests: Test ch8_deadlock_sem1 in Process 17 exited with code 0
Usertests: Test ch8_deadlock_sem2 in Process 47 exited with code 0



#### 问答题

1.我认为需要回收的资源包括（1）mutex和信号量（2）地址空间，内核栈，进程内核，其他taskcontrolblock可能会在PCB中被引用，不需要分别回收，因为进程内核中有他们的相关引用，如果进程被删除了，那么智能指针销毁这些进程，我们只需要确保kill掉0号线程的时候会把进程kill掉即可

2.第二种实现会尝试判断mutex中是否存在等待状态的任务，如果不存在则进行解锁，这种方式可以避免死锁，即处于锁定状态，但是并没有任务在等待队列中导致所有任务都无法进行。第二种实现发现死锁之后会直接解锁，但是带来的问题就是降低了系统的安全性，有些时候出现死锁应当报错，而不是直接解锁，可能破坏掉一些线程的时间顺序关系。
