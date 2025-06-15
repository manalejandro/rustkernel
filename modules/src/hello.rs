// SPDX-License-Identifier: GPL-2.0

//! Hello World kernel module

#![no_std]
#![no_main]

use kernel::prelude::*;

struct HelloModule {
    message: String,
}

impl kernel::module::Module for HelloModule {
    fn init(_module: &'static kernel::module::ThisModule) -> Result<Self> {
        info!("Hello from Rust kernel module!");
        info!("This is a sample module demonstrating the Rust kernel framework");
        
        Ok(HelloModule {
            message: String::from("Hello, Rust Kernel!"),
        })
    }
    
    fn exit(_module: &'static kernel::module::ThisModule) {
        info!("Goodbye from Hello module");
    }
}

module! {
    type: HelloModule,
    name: "hello_module",
    author: "Rust Kernel Contributors",
    description: "A simple hello world module",
    license: "GPL-2.0",
}
