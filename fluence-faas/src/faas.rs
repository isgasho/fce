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

use super::FaaSError;
use super::faas_interface::FaaSInterface;
use super::faas_interface::FaaSModuleInterface;

use fce::FCE;
use super::IValue;
use fce::FCEModuleConfig;

use std::fs;
use std::path::PathBuf;

pub struct FluenceFaaS {
    process: FCE,

    // names of core modules loaded to FCE
    module_names: Vec<String>,

    // config for code loaded by call_code function
    faas_code_config: FCEModuleConfig,
}

impl FluenceFaaS {
    pub fn new<P: Into<PathBuf>>(
        core_modules_dir: P,
        config_file_path: P,
    ) -> Result<Self, FaaSError> {
        let mut wasm_process = FCE::new();
        let mut module_names = Vec::new();
        let mut core_modules_config = crate::misc::parse_config_from_file(config_file_path.into())?;

        for entry in fs::read_dir(core_modules_dir.into())? {
            let path = entry?.path();
            if path.is_dir() {
                // just skip directories
                continue;
            }

            let module_name = path.file_name().unwrap();
            let module_name = module_name
                .to_os_string()
                .into_string()
                .map_err(|e| FaaSError::IOError(format!("failed to read from {:?} file", e)))?;

            let module_bytes = fs::read(path.clone())?;

            let core_module_config = crate::misc::make_wasm_process_config(
                core_modules_config.modules_config.remove(&module_name),
            )?;
            wasm_process.load_module(module_name.clone(), &module_bytes, core_module_config)?;
            module_names.push(module_name);
        }

        let rpc_module_config =
            crate::misc::make_wasm_process_config(core_modules_config.rpc_module_config)?;

        Ok(Self {
            process: wasm_process,
            module_names,
            faas_code_config: rpc_module_config,
        })
    }

    /// Executes provided Wasm code in the internal environment (with access to module exports).
    pub fn call_code(
        &mut self,
        wasm_rpc: &[u8],
        func_name: &str,
        args: &[IValue],
    ) -> Result<Vec<IValue>, FaaSError> {
        let rpc_module_name = "ipfs_rpc";

        self.process
            .load_module(rpc_module_name, wasm_rpc, self.faas_code_config.clone())?;

        let call_result = self.process.call(rpc_module_name, func_name, args)?;
        self.process.unload_module(rpc_module_name)?;

        Ok(call_result)
    }

    /// Call a specified function of loaded on a startup module by its name.
    pub fn call_module(
        &mut self,
        module_name: &str,
        func_name: &str,
        args: &[IValue],
    ) -> Result<Vec<IValue>, FaaSError> {
        self.process
            .call(module_name, func_name, args)
            .map_err(Into::into)
    }

    /// Return all export functions (name and signatures) of loaded on a startup modules.
    pub fn get_interface(&self) -> FaaSInterface {
        let mut modules = Vec::with_capacity(self.module_names.len());

        for module_name in self.module_names.iter() {
            let functions = self.process.get_interface(module_name).unwrap();
            modules.push(FaaSModuleInterface {
                name: module_name,
                functions,
            })
        }

        FaaSInterface { modules }
    }
}
