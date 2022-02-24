use crate::config::MAX_SYSCALL_NUM;

use super::TaskContext;

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub syscall_ids: [usize; MAX_SYSCALL_NUM],
    pub syscall_times: [usize; MAX_SYSCALL_NUM],
    pub task_current_time: usize,
    pub task_time: usize,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
