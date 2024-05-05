//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use super::{__switch, TaskInfo};
use super::{fetch_task, TaskStatus, fetch_min_task};
use super::{TaskContext, TaskControlBlock};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use crate::timer::get_time_ms;
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
    pub fn refresh_task_info(&self, syscall_idx: usize, refresh_flag: bool){
        let inner = self.current().unwrap();
        let mut current_task = inner.inner_exclusive_access();
        let status = current_task.task_status;
        current_task.task_info.set_status(status);
        if refresh_flag {
            current_task.task_info.increase_syscall_time(syscall_idx);
        }
    }

    pub fn get_current_task_info(&self) ->TaskInfo{
        let inner = self.current().unwrap();
        let mut current_task = inner.inner_exclusive_access();
        
        let run_time = get_time_ms() - current_task.task_start_time;
        current_task.task_info.set_run_time(run_time);

        let current_task_info = current_task.task_info;
        current_task_info
    }

    pub fn current_task_m_map(&self, start: usize, len:usize, port: usize) -> isize{

        let inner = self.current().unwrap();
        let mut current_task = inner.inner_exclusive_access();

        current_task.m_map(start, len, port)

    }

    pub fn current_task_m_unmap(&self, start: usize, len:usize) -> isize{
        let inner = self.current().unwrap();
        let mut current_task = inner.inner_exclusive_access();

        current_task.m_unmap(start, len)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_min_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

pub fn get_current_processor_info() -> TaskInfo {
    PROCESSOR.exclusive_access().get_current_task_info()
}

pub fn refresh_processor_syscall_times(syscall: usize){
    PROCESSOR.exclusive_access().refresh_task_info(syscall, true);
}

pub fn current_processor_m_map(start: usize, len: usize, port: usize) -> isize {
    PROCESSOR.exclusive_access().current_task_m_map(start, len, port)
}

pub fn current_processor_m_unmap(start: usize, len: usize) -> isize {
    PROCESSOR.exclusive_access().current_task_m_unmap(start, len)
}