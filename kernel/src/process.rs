// SPDX-License-Identifier: GPL-2.0

//! Process and thread management

use crate::types::{Pid, Tid, Uid, Gid};
use crate::error::{Error, Result};
use crate::sync::Spinlock;
use crate::memory::VirtAddr;
use alloc::{string::String, vec::Vec, collections::BTreeMap};
use core::sync::atomic::{AtomicU32, Ordering};

/// Process state - compatible with Linux kernel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Dead,
}

/// Process structure - similar to Linux task_struct
#[derive(Debug)]
pub struct Process {
    pub pid: Pid,
    pub parent: Option<Pid>,
    pub state: ProcessState,
    pub uid: Uid,
    pub gid: Gid,
    pub name: String,
    pub threads: Vec<Thread>,
    pub memory_map: Option<VirtAddr>, // Points to mm_struct equivalent
    pub files: Vec<u32>, // File descriptor table
    pub signal_pending: bool,
    pub exit_code: i32,
}

impl Process {
    pub fn new(pid: Pid, name: String, uid: Uid, gid: Gid) -> Self {
        Self {
            pid,
            parent: None,
            state: ProcessState::Running,
            uid,
            gid,
            name,
            threads: Vec::new(),
            memory_map: None,
            files: Vec::new(),
            signal_pending: false,
            exit_code: 0,
        }
    }
    
    /// Add a thread to this process
    pub fn add_thread(&mut self, thread: Thread) {
        self.threads.push(thread);
    }
    
    /// Get the main thread
    pub fn main_thread(&self) -> Option<&Thread> {
        self.threads.first()
    }
    
    /// Set process state
    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }
    
    /// Check if process is running
    pub fn is_running(&self) -> bool {
        self.state == ProcessState::Running
    }
}

/// Thread structure - Linux-compatible
#[derive(Debug, Clone)]
pub struct Thread {
    pub tid: Tid,
    pub process_pid: Pid,
    pub state: ProcessState,
    pub stack_pointer: VirtAddr,
    pub instruction_pointer: VirtAddr,
    pub priority: i32,
    pub nice: i32, // Nice value (-20 to 19)
    pub cpu_time: u64, // Nanoseconds
    pub context: ThreadContext,
}

impl Thread {
    pub fn new(tid: Tid, process_pid: Pid, priority: i32) -> Self {
        Self {
            tid,
            process_pid,
            state: ProcessState::Running,
            stack_pointer: VirtAddr::new(0),
            instruction_pointer: VirtAddr::new(0),
            priority,
            nice: 0,
            cpu_time: 0,
            context: ThreadContext::new(),
        }
    }
    
    /// Set thread state
    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }
    
    /// Update CPU time
    pub fn add_cpu_time(&mut self, time: u64) {
        self.cpu_time += time;
    }
}

/// Thread context for context switching
#[derive(Debug, Clone, Default)]
pub struct ThreadContext {
    // x86_64 registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
    // Segment registers
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
    pub ss: u16,
}

impl ThreadContext {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Global process table
static PROCESS_TABLE: Spinlock<ProcessTable> = Spinlock::new(ProcessTable::new());
static NEXT_PID: AtomicU32 = AtomicU32::new(1);
static NEXT_TID: AtomicU32 = AtomicU32::new(1);

/// Process table implementation
struct ProcessTable {
    processes: BTreeMap<Pid, Process>,
    current_process: Option<Pid>,
}

impl ProcessTable {
    const fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            current_process: None,
        }
    }
    
    fn add_process(&mut self, process: Process) {
        let pid = process.pid;
        self.processes.insert(pid, process);
        if self.current_process.is_none() {
            self.current_process = Some(pid);
        }
    }
    
    fn get_process(&self, pid: Pid) -> Option<&Process> {
        self.processes.get(&pid)
    }
    
    fn get_process_mut(&mut self, pid: Pid) -> Option<&mut Process> {
        self.processes.get_mut(&pid)
    }
    
    fn remove_process(&mut self, pid: Pid) -> Option<Process> {
        let process = self.processes.remove(&pid);
        if self.current_process == Some(pid) {
            self.current_process = self.processes.keys().next().copied();
        }
        process
    }
    
    fn list_processes(&self) -> Vec<Pid> {
        self.processes.keys().copied().collect()
    }
}

/// Allocate a new PID
pub fn allocate_pid() -> Pid {
    Pid(NEXT_PID.fetch_add(1, Ordering::SeqCst))
}

/// Allocate a new TID
pub fn allocate_tid() -> Tid {
    Tid(NEXT_TID.fetch_add(1, Ordering::SeqCst))
}

/// Create a new process
pub fn create_process(name: String, uid: Uid, gid: Gid) -> Result<Pid> {
    let pid = allocate_pid();
    let mut process = Process::new(pid, name, uid, gid);
    
    // Create main thread
    let tid = allocate_tid();
    let main_thread = Thread::new(tid, pid, 0);
    process.add_thread(main_thread);
    
    let mut table = PROCESS_TABLE.lock();
    table.add_process(process);
    
    Ok(pid)
}

/// Get current process
pub fn current_process() -> Option<Pid> {
    let table = PROCESS_TABLE.lock();
    table.current_process
}

/// Get process by PID
pub fn get_process(pid: Pid) -> Option<Process> {
    let table = PROCESS_TABLE.lock();
    table.get_process(pid).cloned()
}

/// Kill a process
pub fn kill_process(pid: Pid, signal: i32) -> Result<()> {
    let mut table = PROCESS_TABLE.lock();
    if let Some(process) = table.get_process_mut(pid) {
        process.set_state(ProcessState::Dead);
        process.exit_code = signal;
        Ok(())
    } else {
        Err(Error::NotFound)
    }
}

/// List all processes
pub fn list_processes() -> Vec<Pid> {
    let table = PROCESS_TABLE.lock();
    table.list_processes()
}

/// Initialize the process subsystem
pub fn init() -> Result<()> {
    // Create kernel process (PID 0)
    let _kernel_pid = create_process(
        String::from("kernel"),
        Uid(0),
        Gid(0)
    )?;
    
    // Create init process (PID 1) 
    let _init_pid = create_process(
        String::from("init"),
        Uid(0), 
        Gid(0)
    )?;
    
    Ok(())
}
