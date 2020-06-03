/*
 * Copyright 2020 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#![deny(
    nonstandard_style,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_unsafe,
    unreachable_patterns
)]
#![warn(rust_2018_idioms)]

mod misc;
/// Command-line tool intended to test FCE VM.
mod vm;

use crate::misc::SlicePrettyPrinter;
use crate::vm::config::Config;
use crate::vm::fce::FCE;
use crate::vm::service::FCEService;

use exitfailure::ExitFailure;
use std::fs;

fn main() -> Result<(), ExitFailure> {
    println!("Welcome to the FCE CLI:");
    let mut rl = rustyline::Editor::<()>::new();
    let mut fce = FCE::new();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                // TODO: improve argument parsing
                let cmd: Vec<_> = line.split(' ').collect();
                match cmd[0] {
                    "add" => {
                        let module_name = cmd[1].to_string();
                        let wasm_bytes = fs::read(cmd[2]);
                        if let Err(e) = wasm_bytes {
                            println!("failed to read wasm module: {}", e);
                            continue;
                        }

                        let config = Config::default();
                        let result_msg =
                            match fce.register_module(module_name, &wasm_bytes.unwrap(), config) {
                                Ok(_) => "module successfully registered in FCE".to_string(),
                                Err(e) => format!("module registration failed with: {:?}", e),
                            };
                        println!("{}", result_msg);
                    }
                    "del" => {
                        let module_name = cmd[1];
                        let result_msg = match fce.unregister_module(module_name) {
                            Ok(_) => "module successfully deleted from FCE".to_string(),
                            Err(e) => format!("module deletion failed with: {:?}", e),
                        };
                        println!("{}", result_msg);
                    }
                    "execute" => {
                        let module_name = cmd[1];
                        let arg = cmd[2..].join(" ");
                        let result = match fce.invoke(module_name, arg.as_bytes()) {
                            Ok(result) => {
                                let outcome_copy = result.outcome.clone();
                                match String::from_utf8(result.outcome) {
                                    Ok(s) => format!("result: {}", s),
                                    Err(_) => format!("result: {:?}", outcome_copy),
                                }
                            }
                            Err(e) => format!("execution failed with {:?}", e),
                        };
                        println!("{}", result);
                    }
                    "hash" => {
                        let hash = fce.compute_state_hash();
                        println!(
                            "vm state hash is {:2x}",
                            SlicePrettyPrinter(hash.as_slice())
                        );
                    }
                    "help" => {
                        println!(
                            "Enter:\n\
                                add <module_name> <module_path> - to add a new Wasm module to FCE\n\
                                del <module_name>               - to delete Wasm module to FCE\n\
                                execute <module_name> <arg>     - to call invoke on module with module_name\n\
                                hash                            - to compute hash of internal Wasm state\n\
                                help                            - to print this message\n\
                                e/exit/q/quit                   - to exit"
                        );
                    }
                    "e" | "exit" | "q" | "quit" => break,
                    _ => {
                        println!("unsupported command");
                    }
                }
            }
            Err(e) => {
                println!("a error occurred: {}", e);
                break;
            }
        }
    }

    Ok(())
}