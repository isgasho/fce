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

use wasmer_wit::errors::InstructionError;
use wasmer_runtime::error::{
    CallError, CompileError, CreationError, Error as WasmerError, ResolveError, RuntimeError,
};

use std::error::Error;

#[derive(Debug)]
pub enum FCEError {
    /// This error type is produced by Wasmer during resolving a Wasm function.
    WasmerResolveError(String),

    /// Error related to calling a main Wasm module.
    WasmerInvokeError(String),

    /// Error that raises during compilation Wasm code by Wasmer.
    WasmerCreationError(String),

    /// Error that raises during creation of some Wasm objects (like table and memory) by Wasmer.
    WasmerCompileError(String),

    /// Error that raises on the preparation step.
    PrepareError(String),

    /// Indicates that there is already a module with such name.
    NonUniqueModuleName,

    /// Returns when there is no module with such name.
    NoSuchFunction(String),

    /// Returns when there is no module with such name.
    NoSuchModule,

    /// WIT section is absent.
    NoWITSection,

    /// Multiple WIT sections.
    MultipleWITSections,

    /// WIT section remainder isn't empty.
    WITRemainderNotEmpty,

    /// An error occurred while parsing WIT section.
    WITParseError,
}

impl Error for FCEError {}

impl std::fmt::Display for FCEError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            FCEError::WasmerResolveError(msg) => write!(f, "WasmerResolveError: {}", msg),
            FCEError::WasmerInvokeError(msg) => write!(f, "WasmerInvokeError: {}", msg),
            FCEError::WasmerCompileError(msg) => write!(f, "WasmerCompileError: {}", msg),
            FCEError::WasmerCreationError(msg) => write!(f, "WasmerCreationError: {}", msg),
            FCEError::PrepareError(msg) => {
                write!(f, "Prepare error: {}, probably module is mailformed", msg)
            }
            FCEError::NonUniqueModuleName => write!(f, "FCE already has module with such a name"),
            FCEError::NoSuchFunction(msg) => {
                write!(f, "FCE doesn't have a function with such a name: {}", msg)
            }
            FCEError::NoSuchModule => write!(f, "FCE doesn't have a module with such a name"),
            FCEError::NoWITSection => write!(
                f,
                "Loaded module doesn't contain WIT section that is neccessary for instantiation"
            ),
            FCEError::MultipleWITSections => write!(
                f,
                "Loaded module contains multiple WIT sections that is unsupported now"
            ),
            FCEError::WITRemainderNotEmpty => write!(
                f,
                "WIT section remainder isn't empty - WIT section possibly corrupted"
            ),
            FCEError::WITParseError => write!(f, "WIT section is corrupted"),
        }
    }
}

impl From<CreationError> for FCEError {
    fn from(err: CreationError) -> Self {
        FCEError::WasmerCreationError(format!("{}", err))
    }
}

impl From<CompileError> for FCEError {
    fn from(err: CompileError) -> Self {
        FCEError::WasmerCompileError(format!("{}", err))
    }
}

impl From<parity_wasm::elements::Error> for FCEError {
    fn from(err: parity_wasm::elements::Error) -> Self {
        FCEError::PrepareError(format!("{}", err))
    }
}

impl From<CallError> for FCEError {
    fn from(err: CallError) -> Self {
        match err {
            CallError::Resolve(err) => FCEError::WasmerResolveError(format!("{}", err)),
            CallError::Runtime(err) => FCEError::WasmerInvokeError(format!("{}", err)),
        }
    }
}

impl From<ResolveError> for FCEError {
    fn from(err: ResolveError) -> Self {
        FCEError::WasmerResolveError(format!("{}", err))
    }
}

impl From<RuntimeError> for FCEError {
    fn from(err: RuntimeError) -> Self {
        FCEError::WasmerInvokeError(format!("{}", err))
    }
}

impl From<WasmerError> for FCEError {
    fn from(err: WasmerError) -> Self {
        FCEError::WasmerInvokeError(format!("{}", err))
    }
}

impl From<InstructionError> for FCEError {
    fn from(err: InstructionError) -> Self {
        FCEError::WasmerInvokeError(format!("{}", err))
    }
}