use crate::task::{
    exit_current_and_run_next, get_task_info, suspend_current_and_run_next, TaskInfo, TaskStatus,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let info = get_task_info();
    println!("{:#?}", info);
    unsafe {
        *ti = TaskInfo {
            id: info.id,
            status: info.status,
            syscall_ids: info.syscall_ids,
            syscall_times: info.syscall_times,
            time: info.time,
        }
    }
    0
}
