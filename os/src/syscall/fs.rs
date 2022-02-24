use crate::{
    config::{APP_BASE_ADDRESS, APP_SIZE_LIMIT},
    task,
};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let task_id = task::get_current_task_id();
    let start = buf as usize;
    let base_addr = APP_BASE_ADDRESS + task_id * APP_SIZE_LIMIT;
    if start < base_addr || start + len > base_addr + APP_SIZE_LIMIT {
        return -1;
    }
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            error!("{}", "Unsupported fd in sys_write!");
            -1
        }
    }
}
