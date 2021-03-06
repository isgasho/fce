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

use super::wit_prelude::*;
use super::fce_module::FCEModule;
use super::IRecordType;
use crate::Result;

use fce_wit_interfaces::FCEWITInterfaces;
use fce_wit_interfaces::WITAstType;
use wasmer_wit::interpreter::wasm;
use wasmer_wit::interpreter::wasm::structures::{LocalImportIndex, TypedIndex};
use wasmer_core::Instance as WasmerInstance;

use std::collections::HashMap;
use std::rc::Rc;

pub type RecordTypes = HashMap<u64, Rc<IRecordType>>;

/// Contains all import and export functions that could be called from WIT context by call-core.
#[derive(Clone)]
pub(super) struct WITInstance {
    /// WIT functions indexed by id.
    funcs: HashMap<usize, WITFunction>,

    /// WIT memories.
    memories: Vec<WITMemory>,

    /// All record types that instance contains.
    record_types_by_id: RecordTypes,
}

impl WITInstance {
    pub(super) fn new(
        wasmer_instance: &WasmerInstance,
        wit: &FCEWITInterfaces<'_>,
        modules: &HashMap<String, FCEModule>,
    ) -> Result<Self> {
        let mut exports = Self::extract_raw_exports(&wasmer_instance, wit)?;
        let imports = Self::extract_imports(modules, wit, exports.len())?;
        let memories = Self::extract_memories(&wasmer_instance);

        exports.extend(imports);
        let funcs = exports;

        let record_types_by_id = Self::extract_record_types(wit);

        Ok(Self {
            funcs,
            memories,
            record_types_by_id,
        })
    }

    fn extract_raw_exports(
        wasmer_instance: &WasmerInstance,
        wit: &FCEWITInterfaces<'_>,
    ) -> Result<HashMap<usize, WITFunction>> {
        use wasmer_core::DynFunc;

        let module_exports = &wasmer_instance.exports;

        wit.exports()
            .enumerate()
            .map(|(export_id, export)| {
                let export_func = module_exports.get(export.name)?;
                unsafe {
                    // TODO: refactor this with new Wasmer API when it is ready
                    // here it is safe because dyn func is never lives WITInstance
                    let export_func =
                        std::mem::transmute::<DynFunc<'_>, DynFunc<'static>>(export_func);
                    Ok((
                        export_id,
                        WITFunction::from_export(export_func, export.name.to_string())?,
                    ))
                }
            })
            .collect()
    }

    /// Extracts only those imports that don't have implementations.
    fn extract_imports(
        modules: &HashMap<String, FCEModule>,
        wit: &FCEWITInterfaces<'_>,
        start_index: usize,
    ) -> Result<HashMap<usize, WITFunction>> {
        wit.imports()
            .filter(|import|
                // filter out imports that have implementations
                matches!(wit.adapter_types_by_core_type(import.function_type), Some(_)))
            .enumerate()
            .map(|(idx, import)| match modules.get(import.namespace) {
                Some(module) => {
                    use wasmer_wit::ast::Type;
                    let (arguments, output_types) =
                        match wit.type_by_idx_r(import.function_type - 2)? {
                            Type::Function {
                                arguments,
                                output_types,
                            } => (arguments.clone(), output_types.clone()),
                            ty => {
                                return Err(FCEError::IncorrectWIT(format!(
                                    "WIT should has Type::Function, but {:?} met",
                                    ty
                                )))
                            }
                        };

                    let func =
                        WITFunction::from_import(module, import.name, arguments, output_types)?;

                    Ok((start_index + idx as usize, func))
                }
                None => Err(FCEError::NoSuchModule(format!(
                    "trying to get imports from module with name {} that is not loaded",
                    import.namespace
                ))),
            })
            .collect::<Result<HashMap<_, _>>>()
    }

    fn extract_memories(wasmer_instance: &WasmerInstance) -> Vec<WITMemory> {
        use wasmer_core::export::Export::Memory;

        let mut memories = wasmer_instance
            .exports()
            .filter_map(|(_, export)| match export {
                Memory(memory) => Some(WITMemory(memory)),
                _ => None,
            })
            .collect::<Vec<_>>();

        if let Some(Memory(memory)) = wasmer_instance
            .import_object
            .maybe_with_namespace("env", |env| env.get_export("memory"))
        {
            memories.push(WITMemory(memory));
        }

        memories
    }

    fn extract_record_types(wit: &FCEWITInterfaces<'_>) -> RecordTypes {
        let (record_types_by_id, _) = wit.types().fold(
            (HashMap::new(), 0u64),
            |(mut record_types_by_id, id), ty| {
                match ty {
                    WITAstType::Record(record_type) => {
                        record_types_by_id.insert(id, record_type.clone());
                    }
                    WITAstType::Function { .. } => {}
                };
                (record_types_by_id, id + 1)
            },
        );

        record_types_by_id
    }
}

impl wasm::structures::Instance<WITExport, WITFunction, WITMemory, WITMemoryView<'_>>
    for WITInstance
{
    fn export(&self, _export_name: &str) -> Option<&WITExport> {
        // exports aren't used in this version of WIT
        None
    }

    fn local_or_import<I: TypedIndex + LocalImportIndex>(&self, index: I) -> Option<&WITFunction> {
        self.funcs.get(&index.index())
    }

    fn memory(&self, index: usize) -> Option<&WITMemory> {
        if index >= self.memories.len() {
            None
        } else {
            Some(&self.memories[index])
        }
    }

    fn wit_record_by_id(&self, index: u64) -> Option<&Rc<IRecordType>> {
        self.record_types_by_id.get(&index)
    }
}
