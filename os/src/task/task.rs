//! Types related to task management

use crate::config::MAX_SYSCALL_NUM;

use super::TaskContext;

/// Task information
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}
impl TaskInfo{
    pub fn new(ts: TaskStatus)->Self{
        Self { 
            status: ts, 
            //initialize syscall number as zero
            syscall_times: [0; MAX_SYSCALL_NUM], 
            time: 0, 
        }
    }
    pub fn set_status(&mut self, status: TaskStatus){
        self.status = status;
    }

    pub fn increase_syscall_time(&mut self, idx: usize){
        if idx < MAX_SYSCALL_NUM {
            self.syscall_times[idx] += 1;
        }
    }
    pub fn set_run_time(&mut self, t: usize){
        self.time = t;
    }
}

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    pub task_info: TaskInfo,
    pub task_start_time: usize,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
