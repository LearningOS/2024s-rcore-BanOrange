//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,
    },
    timer::get_time_us,
};

use crate::mm::page_table::get_timeval;
use crate::task::{current_user_token,change_info,set_current_ms_mmap,del_current_ms_mmap};
use crate::mm::page_table::get_taskinfo_from_app;
use crate::config::PAGE_SIZE;

const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// sbrk syscall
const SYSCALL_SBRK: usize = 214;
/// munmap syscall
const SYSCALL_MUNMAP: usize = 215;
/// mmap syscall
const SYSCALL_MMAP: usize = 222;
/// taskinfo syscall
const SYSCALL_TASK_INFO: usize = 410;

#[repr(C)]
#[derive(Debug)]
///用於存儲時間
pub struct TimeVal {
    ///秒數
    pub sec: usize,
    ///微秒數
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    change_info(SYSCALL_EXIT);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    change_info(SYSCALL_YIELD);
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    change_info(SYSCALL_GET_TIME);
    let v = get_timeval(current_user_token(),_ts);
    let us = get_time_us();
    v.sec = us / 1_000_000;
    v.usec = us % 1_000_000;
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let v = get_taskinfo_from_app(current_user_token(),_ti);
    let task_info = change_info(SYSCALL_TASK_INFO);
    v.status = task_info.status;
    v.time = task_info.time;
    v.syscall_times = task_info.syscall_times;
    0
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    change_info(SYSCALL_MMAP);

    //先验证一下_port是否符合要求
    if _port & !0x7 != 0 || _port & 0x7 == 0 {
        return -1;
    }
    //在验证一下开始地址是否页对齐
    if _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    //该函数会返回正确与否
    set_current_ms_mmap(_start,_len,_port)
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    change_info(SYSCALL_MUNMAP);
    //验证一下开始地址是否页对齐
    if _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    del_current_ms_mmap(_start,_len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    change_info(SYSCALL_SBRK);
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
