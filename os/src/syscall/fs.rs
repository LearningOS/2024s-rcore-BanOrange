//! File and filesystem-related syscalls
use crate::fs::{open_file, OpenFlags, Stat};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};
use crate::fs::inode::unlinkat;
use crate::fs::inode::linkat;
use crate::fs::inode::fstat;
// use crate::fs::inode::ls;
use crate::mm::page_table::get_stat_from_app;

/// unlinkat syscall
const SYSCALL_UNLINKAT: usize = 35;
/// linkat syscall
const SYSCALL_LINKAT: usize = 37;
/// open syscall
const SYSCALL_OPEN: usize = 56;
/// close syscall
const SYSCALL_CLOSE: usize = 57;
/// read syscall
const SYSCALL_READ: usize = 63;
/// write syscall
const SYSCALL_WRITE: usize = 64;
/// fstat syscall
const SYSCALL_FSTAT: usize = 80;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    current_task().unwrap().change_info(SYSCALL_WRITE);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    current_task().unwrap().change_info(SYSCALL_READ);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    current_task().unwrap().change_info(SYSCALL_OPEN);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    current_task().unwrap().change_info(SYSCALL_CLOSE);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat in build",
        current_task().unwrap().pid.0
    );
    current_task().unwrap().change_info(SYSCALL_FSTAT);
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if _fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[_fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_fstat .. file.stat");
        //对于osinode类型get_info得到的是block_id;
        let (block_id,block_offset) = file.get_info();
        let stat = get_stat_from_app(current_user_token(),_st);
        
        // println!("this is fstat");
        // ls();
        match fstat(block_id,block_offset) {
            Some(stat_disk) => {
                stat.nlink = stat_disk.nlink;
                stat.dev = 0;
                stat.ino = stat_disk.ino;
                stat.mode = stat_disk.mode;
                stat.pad = stat_disk.pad;
                return 0;
            },
            None => {
                return -1;
            }
        }
    } else {
        return -1;
    }
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat in build",
        current_task().unwrap().pid.0
    );
    current_task().unwrap().change_info(SYSCALL_LINKAT);
    let token = current_user_token();
    let old_path = translated_str(token, _old_name);
    let new_path = translated_str(token,_new_name);
    if old_path == new_path {
        return -1;
    }
    linkat(&old_path,&new_path);
    // println!("this is link");
    // ls();
    return 0;

}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat in build",
        current_task().unwrap().pid.0
    );
    current_task().unwrap().change_info(SYSCALL_UNLINKAT);
    let token = current_user_token();
    let name = translated_str(token,_name);
    unlinkat(&name);
    // println!("this is unlink");
    // ls();
    return 0;
}
