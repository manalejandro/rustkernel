// SPDX-License-Identifier: GPL-2.0

//! Process and thread management

use alloc::{
	collections::BTreeMap,
	string::{String, ToString},
	vec::Vec,
};
use core::sync::atomic::{AtomicU32, Ordering};

use crate::arch::x86_64::context::Context;
use crate::error::{Error, Result};
use crate::memory::VirtAddr;
use crate::sync::Spinlock;
use crate::types::{Gid, Pid, Tid, Uid};

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
#[derive(Debug, Clone)]
pub struct Process {
	pub pid: Pid,
	pub parent: Option<Pid>,
	pub state: ProcessState,
	pub uid: Uid,
	pub gid: Gid,
	pub name: String,
	pub threads: Vec<Thread>,
	pub memory_map: Option<VirtAddr>, // Points to mm_struct equivalent
	pub files: Vec<u32>,              // File descriptor table
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

	/// Fork the current process (create a copy)
	pub fn fork(&self) -> Result<Process> {
		let new_pid = allocate_pid();
		let mut child = self.clone();
		child.pid = new_pid;
		child.parent = Some(self.pid);
		child.state = ProcessState::Running;

		// TODO: Copy memory space (copy-on-write)
		// TODO: Copy file descriptor table
		// TODO: Set up new page tables

		Ok(child)
	}

	/// Execute a new program in this process
	pub fn exec(&mut self, program_path: &str, args: Vec<String>) -> Result<()> {
		// TODO: Load program from filesystem
		// TODO: Set up new memory layout
		// TODO: Initialize stack with arguments
		// TODO: Set entry point

		self.name = program_path.to_string();
		Ok(())
	}

	/// Terminate the process with given exit code
	pub fn exit(&mut self, exit_code: i32) {
		self.state = ProcessState::Zombie;
		self.exit_code = exit_code;

		// TODO: Free memory
		// TODO: Close file descriptors
		// TODO: Notify parent
		// TODO: Reparent children to init
	}

	/// Send a signal to the process
	pub fn send_signal(&mut self, signal: i32) -> Result<()> {
		match signal {
			9 => {
				// SIGKILL
				self.state = ProcessState::Dead;
			}
			15 => {
				// SIGTERM
				self.signal_pending = true;
				// TODO: Add to signal queue
			}
			_ => {
				// TODO: Handle other signals
			}
		}
		Ok(())
	}

	/// Wait for child processes
	pub fn wait(&self) -> Result<(Pid, i32)> {
		// TODO: Block until child exits
		// TODO: Return child PID and exit status
		Err(Error::ECHILD)
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
	pub nice: i32,     // Nice value (-20 to 19)
	pub cpu_time: u64, // Nanoseconds
	pub context: Context,
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
			context: Context::new(),
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

/// Global process table
pub static PROCESS_TABLE: Spinlock<ProcessTable> = Spinlock::new(ProcessTable::new());
static NEXT_PID: AtomicU32 = AtomicU32::new(1);
static NEXT_TID: AtomicU32 = AtomicU32::new(1);

/// Process table implementation
pub struct ProcessTable {
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

	pub fn add_process(&mut self, process: Process) {
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

	#[allow(dead_code)]
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

	pub fn find_thread(&self, tid: Tid) -> Option<&Thread> {
		for process in self.processes.values() {
			for thread in &process.threads {
				if thread.tid == tid {
					return Some(thread);
				}
			}
		}
		None
	}

	pub fn find_thread_mut(&mut self, tid: Tid) -> Option<&mut Thread> {
		for process in self.processes.values_mut() {
			for thread in &mut process.threads {
				if thread.tid == tid {
					return Some(thread);
				}
			}
		}
		None
	}

	pub fn find_two_threads_mut(
		&mut self,
		tid1: Tid,
		tid2: Tid,
	) -> (Option<&mut Thread>, Option<&mut Thread>) {
		if tid1 == tid2 {
			let t = self.find_thread_mut(tid1);
			return (t, None);
		}

		// This is a bit inefficient but safe
		// We can't easily return two mutable references to the same structure
		// But since they are in different processes or different threads, they are distinct memory locations.
		// We can use unsafe to cheat the borrow checker, knowing that tid1 != tid2.

		let ptr = self as *mut ProcessTable;
		unsafe {
			let t1 = (*ptr).find_thread_mut(tid1);
			let t2 = (*ptr).find_thread_mut(tid2);
			(t1, t2)
		}
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

/// Add a thread to the kernel process (PID 0)
pub fn add_kernel_thread(tid: Tid, context: Context, stack_pointer: VirtAddr) -> Result<()> {
	let mut table = PROCESS_TABLE.lock();
	if let Some(process) = table.get_process_mut(Pid(0)) {
		let mut thread = Thread::new(tid, Pid(0), 0);
		thread.context = context;
		thread.stack_pointer = stack_pointer;
		process.add_thread(thread);
		Ok(())
	} else {
		Err(Error::NotFound)
	}
}

/// Get current process PID
pub fn current_process_pid() -> Option<Pid> {
	let table = PROCESS_TABLE.lock();
	table.current_process
}

/// Get current process object
pub fn current_process() -> Option<Process> {
	let table = PROCESS_TABLE.lock();
	if let Some(pid) = table.current_process {
		table.get_process(pid).cloned()
	} else {
		None
	}
}

/// Get process by PID
pub fn find_process(pid: Pid) -> Option<Process> {
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

/// Initialize process management
pub fn init_process_management() -> Result<()> {
	init()
}

/// Initialize the process subsystem
pub fn init() -> Result<()> {
	// Initialize the process table and create kernel process (PID 0)
	let kernel_pid = create_process(
		"kernel".to_string(),
		Uid(0), // root
		Gid(0), // root
	)?;

	crate::info!(
		"Process management initialized with kernel PID {}",
		kernel_pid.0
	);
	Ok(())
}
