mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::config::{MAX_APP_NUM, MAX_SYSCALL_NUM};
use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use crate::syscall::SYSCALL_ID_LIST;
use crate::timer::{get_time, get_time_us};
use lazy_static::*;
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

#[derive(Clone, Copy)]
pub struct SyscallInfo(usize, usize);

#[derive(Clone, Copy, Debug)]
pub struct TaskInfo {
    pub id: usize,
    pub status: TaskStatus,
    pub syscall_ids: [usize; MAX_SYSCALL_NUM],
    pub syscall_times: [usize; MAX_SYSCALL_NUM],
    pub time: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
            syscall_ids: [0; MAX_SYSCALL_NUM],
            syscall_times: [0; MAX_SYSCALL_NUM],
            task_current_time: 0,
            task_time: 0,
        }; MAX_APP_NUM];
        for (i, t) in tasks.iter_mut().enumerate().take(num_app) {
            t.task_cx = TaskContext::goto_restore(init_app_cx(i));
            t.task_status = TaskStatus::Ready;
            t.syscall_ids = SYSCALL_ID_LIST;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        task0.task_current_time = get_time_us();
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            let current_time = get_time_us();
            inner.tasks[current].task_time += current_time - inner.tasks[current].task_current_time;
            inner.tasks[next].task_current_time = current_time;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }

    fn inc_syscall_times(&self, id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let syscall_ids = inner.tasks[current].syscall_ids;
        syscall_ids.iter().enumerate().for_each(|(i, &syscall_id)| {
            if syscall_id == id {
                inner.tasks[current].syscall_times[i] += 1;
            }
        });
    }

    fn get_current_task_id(&self) -> usize {
        self.inner.exclusive_access().current_task
    }

    fn get_current_task_info(&self) -> TaskInfo {
        let inner = self.inner.exclusive_access();
        let current_task = inner.tasks[inner.current_task];
        TaskInfo {
            id: inner.current_task,
            status: current_task.task_status,
            syscall_ids: current_task.syscall_ids,
            syscall_times: current_task.syscall_times,
            time: current_task.task_time,
        }
    }
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn inc_syscall_times(id: usize) {
    TASK_MANAGER.inc_syscall_times(id);
}

pub fn get_current_task_id() -> usize {
    TASK_MANAGER.get_current_task_id()
}

pub fn get_task_info() -> TaskInfo {
    TASK_MANAGER.get_current_task_info()
}
