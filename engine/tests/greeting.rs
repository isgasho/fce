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

use fce::FCE;
use fce::IValue;

use once_cell::sync::Lazy;

static GREETING_WASM_BYTES: Lazy<Vec<u8>> = Lazy::new(|| {
    std::fs::read("../examples/greeting/artifacts/greeting.wasm")
        .expect("../examples/greeting/artifacts/greeting.wasm should presence")
});

#[test]
pub fn greeting_basic() {
    let mut fce = FCE::new();
    fce.load_module("greeting", &*GREETING_WASM_BYTES, <_>::default())
        .unwrap_or_else(|e| panic!("can't load a module into FCE: {:?}", e));

    let result1 = fce
        .call(
            "greeting",
            "greeting",
            &[IValue::String(String::from("Fluence"))],
        )
        .unwrap_or_else(|e| panic!("can't invoke greeting: {:?}", e));

    let result2 = fce
        .call("greeting", "greeting", &[IValue::String(String::from(""))])
        .unwrap_or_else(|e| panic!("can't invoke greeting: {:?}", e));

    assert_eq!(result1, vec![IValue::String(String::from("Hi, Fluence"))]);
    assert_eq!(result2, vec![IValue::String(String::from("Hi, "))]);
}

#[test]
// test loading module with the same name twice
pub fn non_unique_module_name() {
    let mut fce = FCE::new();
    let module_name = String::from("greeting");
    fce.load_module(&module_name, &*GREETING_WASM_BYTES, <_>::default())
        .unwrap_or_else(|e| panic!("can't load a module into FCE: {:?}", e));

    let load_result = fce.load_module(&module_name, &*GREETING_WASM_BYTES, <_>::default());
    assert!(load_result.is_err());
    assert!(std::matches!(
        load_result.err().unwrap(),
        fce::FCEError::NonUniqueModuleName(_)
    ));
}

#[test]
#[allow(unused_variables)]
// test calling FCE with non-exist module and function names
pub fn non_exist_module_func() {
    let mut fce = FCE::new();
    fce.load_module("greeting", &*GREETING_WASM_BYTES, <_>::default())
        .unwrap_or_else(|e| panic!("can't load a module into FCE: {:?}", e));

    let non_exist_name = String::from("_");

    let call_result1 = fce.call(
        non_exist_name.as_str(),
        "greeting",
        &[IValue::String(String::from("Fluence"))],
    );

    let call_result2 = fce.call(
        "greeting",
        non_exist_name.as_str(),
        &[IValue::String(String::from("Fluence"))],
    );

    let call_result3 = fce.call(
        non_exist_name.as_str(),
        non_exist_name.as_str(),
        &[IValue::String(String::from("Fluence"))],
    );

    assert!(call_result1.is_err());
    assert!(matches!(
        call_result1.err().unwrap(),
        fce::FCEError::NoSuchModule(non_exist_name)
    ));

    assert!(call_result2.is_err());
    assert!(matches!(
        call_result2.err().unwrap(),
        fce::FCEError::NoSuchFunction(non_exist_name)
    ));

    assert!(call_result3.is_err());
    // at first, the module name should be checked
    assert!(matches!(
        call_result3.err().unwrap(),
        fce::FCEError::NoSuchModule(non_exist_name)
    ));
}
