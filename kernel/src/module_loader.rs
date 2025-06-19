// SPDX-License-Identifier: GPL-2.0

//! Dynamic module loading system

use alloc::{
	collections::BTreeMap,
	string::{String, ToString},
	vec::Vec,
};

use crate::error::Result;
use crate::sync::Spinlock;
use crate::{error, info, warn};

/// Module state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
	Loading,
	Live,
	Coming,
	Going,
	Unloading,
}

/// Module descriptor
#[derive(Debug)]
pub struct Module {
	pub name: String,
	pub version: String,
	pub description: String,
	pub state: ModuleState,
	pub init_fn: Option<fn() -> Result<()>>,
	pub cleanup_fn: Option<fn()>,
	pub reference_count: u32,
	pub dependencies: Vec<String>,
}

impl Module {
	pub fn new(name: String, version: String, description: String) -> Self {
		Self {
			name,
			version,
			description,
			state: ModuleState::Loading,
			init_fn: None,
			cleanup_fn: None,
			reference_count: 0,
			dependencies: Vec::new(),
		}
	}

	/// Set module functions
	pub fn set_functions(&mut self, init_fn: fn() -> Result<()>, cleanup_fn: fn()) {
		self.init_fn = Some(init_fn);
		self.cleanup_fn = Some(cleanup_fn);
	}

	/// Add dependency
	pub fn add_dependency(&mut self, dep: String) {
		self.dependencies.push(dep);
	}

	/// Initialize the module
	pub fn init(&mut self) -> Result<()> {
		if let Some(init_fn) = self.init_fn {
			self.state = ModuleState::Coming;
			init_fn()?;
			self.state = ModuleState::Live;
			info!("Module {} initialized successfully", self.name);
			Ok(())
		} else {
			warn!("Module {} has no init function", self.name);
			self.state = ModuleState::Live;
			Ok(())
		}
	}

	/// Cleanup the module
	pub fn cleanup(&mut self) {
		if self.reference_count > 0 {
			warn!(
				"Module {} still has {} references",
				self.name, self.reference_count
			);
			return;
		}

		self.state = ModuleState::Going;
		if let Some(cleanup_fn) = self.cleanup_fn {
			cleanup_fn();
		}
		self.state = ModuleState::Unloading;
		info!("Module {} cleaned up", self.name);
	}
}

/// Module subsystem
static MODULE_SUBSYSTEM: Spinlock<ModuleSubsystem> = Spinlock::new(ModuleSubsystem::new());

struct ModuleSubsystem {
	modules: BTreeMap<String, Module>,
	load_order: Vec<String>,
}

impl ModuleSubsystem {
	const fn new() -> Self {
		Self {
			modules: BTreeMap::new(),
			load_order: Vec::new(),
		}
	}

	fn register_module(&mut self, mut module: Module) -> Result<()> {
		// Check dependencies
		for dep in &module.dependencies {
			if !self.modules.contains_key(dep) {
				error!("Module {} dependency {} not loaded", module.name, dep);
				return Err(crate::error::Error::NotFound);
			}

			// Increment reference count of dependency
			if let Some(dep_module) = self.modules.get_mut(dep) {
				dep_module.reference_count += 1;
			}
		}

		let name = module.name.clone();

		// Initialize the module
		module.init()?;

		// Add to registry
		self.modules.insert(name.clone(), module);
		self.load_order.push(name);

		Ok(())
	}

	fn unload_module(&mut self, name: &str) -> Result<()> {
		if let Some(mut module) = self.modules.remove(name) {
			// Decrement reference counts of dependencies
			for dep in &module.dependencies {
				if let Some(dep_module) = self.modules.get_mut(dep) {
					if dep_module.reference_count > 0 {
						dep_module.reference_count -= 1;
					}
				}
			}

			// Cleanup module
			module.cleanup();

			// Remove from load order
			self.load_order.retain(|n| n != name);

			info!("Module {} unloaded", name);
			Ok(())
		} else {
			error!("Module {} not found", name);
			Err(crate::error::Error::NotFound)
		}
	}

	fn get_module(&self, name: &str) -> Option<&Module> {
		self.modules.get(name)
	}

	fn list_modules(&self) -> Vec<&Module> {
		self.modules.values().collect()
	}
}

/// Initialize module subsystem
pub fn init_modules() -> Result<()> {
	info!("Initializing module subsystem");

	// Register built-in modules
	register_builtin_modules()?;

	info!("Module subsystem initialized");
	Ok(())
}

/// Register a module
pub fn register_module(module: Module) -> Result<()> {
	let mut subsystem = MODULE_SUBSYSTEM.lock();
	subsystem.register_module(module)
}

/// Unload a module
pub fn unload_module(name: &str) -> Result<()> {
	let mut subsystem = MODULE_SUBSYSTEM.lock();
	subsystem.unload_module(name)
}

/// Get module information
pub fn get_module_info(name: &str) -> Option<(String, String, String, ModuleState)> {
	let subsystem = MODULE_SUBSYSTEM.lock();
	if let Some(module) = subsystem.get_module(name) {
		Some((
			module.name.clone(),
			module.version.clone(),
			module.description.clone(),
			module.state,
		))
	} else {
		None
	}
}

/// List all modules
pub fn list_modules() -> Vec<(String, String, String, ModuleState, u32)> {
	let subsystem = MODULE_SUBSYSTEM.lock();
	subsystem
		.list_modules()
		.into_iter()
		.map(|m| {
			(
				m.name.clone(),
				m.version.clone(),
				m.description.clone(),
				m.state,
				m.reference_count,
			)
		})
		.collect()
}

/// Register built-in modules
fn register_builtin_modules() -> Result<()> {
	// Test module
	let mut test_module = Module::new(
		"test".to_string(),
		"1.0.0".to_string(),
		"Test module for demonstration".to_string(),
	);
	test_module.set_functions(test_module_init, test_module_cleanup);
	register_module(test_module)?;

	// Console module
	let mut console_module = Module::new(
		"console".to_string(),
		"1.0.0".to_string(),
		"Console output module".to_string(),
	);
	console_module.set_functions(console_module_init, console_module_cleanup);
	register_module(console_module)?;

	// Network module (depends on console)
	let mut network_module = Module::new(
		"network".to_string(),
		"1.0.0".to_string(),
		"Basic networking module".to_string(),
	);
	network_module.add_dependency("console".to_string());
	network_module.set_functions(network_module_init, network_module_cleanup);
	register_module(network_module)?;

	Ok(())
}

// Built-in module functions
fn test_module_init() -> Result<()> {
	info!("Test module loaded");
	Ok(())
}

fn test_module_cleanup() {
	info!("Test module unloaded");
}

fn console_module_init() -> Result<()> {
	info!("Console module loaded");
	Ok(())
}

fn console_module_cleanup() {
	info!("Console module unloaded");
}

fn network_module_init() -> Result<()> {
	info!("Network module loaded");
	Ok(())
}

fn network_module_cleanup() {
	info!("Network module unloaded");
}

/// Test module loading functionality
pub fn test_module_system() -> Result<()> {
	info!("Testing module system");

	// Create and load a test module
	let mut dynamic_test = Module::new(
		"dynamic_test".to_string(),
		"0.1.0".to_string(),
		"Dynamic test module".to_string(),
	);
	dynamic_test.set_functions(
		|| {
			info!("Dynamic test module init");
			Ok(())
		},
		|| {
			info!("Dynamic test module cleanup");
		},
	);

	register_module(dynamic_test)?;

	// List modules
	let modules = list_modules();
	info!("Loaded modules:");
	for (name, version, desc, state, refs) in modules {
		info!(
			"  {} v{}: {} (state: {:?}, refs: {})",
			name, version, desc, state, refs
		);
	}

	// Unload the test module
	unload_module("dynamic_test")?;

	info!("Module system test completed");
	Ok(())
}
