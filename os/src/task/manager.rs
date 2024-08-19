//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }

    //用以得到當前stride值最小的任務
    pub fn get_smallest_stride_task(&mut self) -> Option<Arc<TaskControlBlock>>{
        let first_task = self.ready_queue.get(0).unwrap();
        let mut min_stride = first_task.inner_exclusive_access().stride; 
        let mut id = 0;
        let mut count =0;
        for item in self.ready_queue.iter() {
            if item.inner_exclusive_access().stride<min_stride{
                min_stride = item.inner_exclusive_access().stride;
                id = count;
            }
            count += 1;
        }
        self.ready_queue.remove(id)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}

//根據stride值找到對應的任務
pub fn fetch_task_stride() -> Option<Arc<TaskControlBlock>>{
    TASK_MANAGER.exclusive_access().get_smallest_stride_task()
}
