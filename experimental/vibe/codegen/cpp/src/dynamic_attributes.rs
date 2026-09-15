// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use quote::quote;

use crate::common::pretty;
use crate::{GenerateError, GeneratedFile, Options};

pub(crate) fn file(
    options: &Options,
    runtime: &syn::Path,
    dynamic: &syn::Path,
) -> Result<GeneratedFile, GenerateError> {
    let namespace = syn::LitStr::new(
        &format!("{}::detail", options.namespace),
        proc_macro2::Span::call_site(),
    );
    let tokens = quote! {
        #[derive(Debug)]
        pub struct DynamicAttributesStorage(#runtime::DynamicAttributes);
        #[derive(Debug)]
        pub struct DynamicListStorage(#dynamic::DynamicList);

        #[cxx::bridge(namespace = #namespace)]
        pub mod ffi {
            unsafe extern "C++" {
                include!("rust/cxx.h");
            }

            #[derive(Debug, Default)]
            pub struct DynamicAttributes {
                // The vector keeps this shared CXX type default-constructible. Converted values
                // contain exactly one opaque storage element.
                pub storage: Vec<Box<DynamicAttributesStorage>>,
            }

            extern "Rust" {
                type DynamicAttributesStorage;
                type DynamicListStorage;

                fn dynamic_attributes_new() -> Box<DynamicAttributesStorage>;
                fn dynamic_attributes_add_null(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                );
                fn dynamic_attributes_add_u8(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: u8,
                );
                fn dynamic_attributes_add_u16(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: u16,
                );
                fn dynamic_attributes_add_u32(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: u32,
                );
                fn dynamic_attributes_add_u64(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: u64,
                );
                fn dynamic_attributes_add_i8(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: i8,
                );
                fn dynamic_attributes_add_i16(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: i16,
                );
                fn dynamic_attributes_add_i32(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: i32,
                );
                fn dynamic_attributes_add_i64(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: i64,
                );
                fn dynamic_attributes_add_f32(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: f32,
                );
                fn dynamic_attributes_add_f64(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: f64,
                );
                fn dynamic_attributes_add_string(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: String,
                );
                fn dynamic_attributes_add_bool(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: bool,
                );
                fn dynamic_attributes_add_structure(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: Box<DynamicAttributesStorage>,
                );
                fn dynamic_attributes_add_u8_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<u8>,
                );
                fn dynamic_attributes_add_u16_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<u16>,
                );
                fn dynamic_attributes_add_u32_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<u32>,
                );
                fn dynamic_attributes_add_u64_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<u64>,
                );
                fn dynamic_attributes_add_i8_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<i8>,
                );
                fn dynamic_attributes_add_i16_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<i16>,
                );
                fn dynamic_attributes_add_i32_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<i32>,
                );
                fn dynamic_attributes_add_i64_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<i64>,
                );
                fn dynamic_attributes_add_f32_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<f32>,
                );
                fn dynamic_attributes_add_f64_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<f64>,
                );
                fn dynamic_attributes_add_string_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<String>,
                );
                fn dynamic_attributes_add_struct_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    values: Vec<Box<DynamicAttributesStorage>>,
                );
                fn dynamic_attributes_add_list(
                    self: &mut DynamicAttributesStorage,
                    key: String,
                    value: Box<DynamicListStorage>,
                );

                fn dynamic_list_empty() -> Box<DynamicListStorage>;
                fn dynamic_list_u8(values: Vec<u8>) -> Box<DynamicListStorage>;
                fn dynamic_list_u16(values: Vec<u16>) -> Box<DynamicListStorage>;
                fn dynamic_list_u32(values: Vec<u32>) -> Box<DynamicListStorage>;
                fn dynamic_list_u64(values: Vec<u64>) -> Box<DynamicListStorage>;
                fn dynamic_list_i8(values: Vec<i8>) -> Box<DynamicListStorage>;
                fn dynamic_list_i16(values: Vec<i16>) -> Box<DynamicListStorage>;
                fn dynamic_list_i32(values: Vec<i32>) -> Box<DynamicListStorage>;
                fn dynamic_list_i64(values: Vec<i64>) -> Box<DynamicListStorage>;
                fn dynamic_list_f32(values: Vec<f32>) -> Box<DynamicListStorage>;
                fn dynamic_list_f64(values: Vec<f64>) -> Box<DynamicListStorage>;
                fn dynamic_list_string(values: Vec<String>) -> Box<DynamicListStorage>;
                fn dynamic_list_structures(
                    values: Vec<Box<DynamicAttributesStorage>>,
                ) -> Box<DynamicListStorage>;
                fn dynamic_list_list(
                    values: Vec<Box<DynamicListStorage>>,
                ) -> Box<DynamicListStorage>;

                fn dynamic_attributes_vec_noop(value: &Vec<DynamicAttributes>);
            }
        }

        fn dynamic_attributes_new() -> Box<DynamicAttributesStorage> {
            Box::new(DynamicAttributesStorage(#runtime::DynamicAttributes::new()))
        }

        impl DynamicAttributesStorage {
            fn dynamic_attributes_add_null(&mut self, key: String) {
                self.0.add(key, #dynamic::DynamicNull);
            }

            fn dynamic_attributes_add_u8(&mut self, key: String, value: u8) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_u16(&mut self, key: String, value: u16) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_u32(&mut self, key: String, value: u32) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_u64(&mut self, key: String, value: u64) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_i8(&mut self, key: String, value: i8) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_i16(&mut self, key: String, value: i16) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_i32(&mut self, key: String, value: i32) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_i64(&mut self, key: String, value: i64) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_f32(&mut self, key: String, value: f32) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_f64(&mut self, key: String, value: f64) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_string(&mut self, key: String, value: String) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_bool(&mut self, key: String, value: bool) {
                self.0.add(key, value);
            }

            fn dynamic_attributes_add_structure(
                &mut self,
                key: String,
                value: Box<DynamicAttributesStorage>,
            ) {
                let DynamicAttributesStorage(value) = *value;
                self.0
                    .add(key, #dynamic::DynamicStruct(value.into_vec()));
            }

            fn dynamic_attributes_add_u8_list(&mut self, key: String, values: Vec<u8>) {
                self.0.add(key, values);
            }

            fn dynamic_attributes_add_u16_list(&mut self, key: String, values: Vec<u16>) {
                self.0.add(key, values);
            }

            fn dynamic_attributes_add_u32_list(&mut self, key: String, values: Vec<u32>) {
                self.0.add(key, values);
            }

            fn dynamic_attributes_add_u64_list(&mut self, key: String, values: Vec<u64>) {
                self.0.add(key, values);
            }

            fn dynamic_attributes_add_i8_list(&mut self, key: String, values: Vec<i8>) {
                self.0.add(key, values);
            }

            fn dynamic_attributes_add_i16_list(&mut self, key: String, values: Vec<i16>) {
                self.0.add(key, values);
            }

            fn dynamic_attributes_add_i32_list(&mut self, key: String, values: Vec<i32>) {
                self.0.add(key, values);
            }

            fn dynamic_attributes_add_i64_list(&mut self, key: String, values: Vec<i64>) {
                self.0.add(key, values);
            }

            fn dynamic_attributes_add_f32_list(&mut self, key: String, values: Vec<f32>) {
                self.0.add(key, values);
            }

            fn dynamic_attributes_add_f64_list(&mut self, key: String, values: Vec<f64>) {
                self.0.add(key, values);
            }

            fn dynamic_attributes_add_string_list(&mut self, key: String, values: Vec<String>) {
                self.0.add(key, values);
            }

            fn dynamic_attributes_add_struct_list(
                &mut self,
                key: String,
                values: Vec<Box<DynamicAttributesStorage>>,
            ) {
                self.0.add(
                    key,
                    values
                        .into_iter()
                        .map(|value| {
                            let DynamicAttributesStorage(value) = *value;
                            #dynamic::DynamicStruct(value.into_vec())
                        })
                        .collect::<Vec<_>>(),
                );
            }

            fn dynamic_attributes_add_list(
                &mut self,
                key: String,
                value: Box<DynamicListStorage>,
            ) {
                self.0.add(key, value.0);
            }
        }

        fn dynamic_list_empty() -> Box<DynamicListStorage> {
            Box::new(DynamicListStorage(#dynamic::DynamicList::List(Vec::new())))
        }

        macro_rules! dynamic_list_constructor {
            ($name:ident, $ty:ty, $variant:ident) => {
                fn $name(values: Vec<$ty>) -> Box<DynamicListStorage> {
                    Box::new(DynamicListStorage(#dynamic::DynamicList::$variant(values)))
                }
            };
        }

        dynamic_list_constructor!(dynamic_list_u8, u8, U8);
        dynamic_list_constructor!(dynamic_list_u16, u16, U16);
        dynamic_list_constructor!(dynamic_list_u32, u32, U32);
        dynamic_list_constructor!(dynamic_list_u64, u64, U64);
        dynamic_list_constructor!(dynamic_list_i8, i8, I8);
        dynamic_list_constructor!(dynamic_list_i16, i16, I16);
        dynamic_list_constructor!(dynamic_list_i32, i32, I32);
        dynamic_list_constructor!(dynamic_list_i64, i64, I64);
        dynamic_list_constructor!(dynamic_list_f32, f32, F32);
        dynamic_list_constructor!(dynamic_list_f64, f64, F64);
        dynamic_list_constructor!(dynamic_list_string, String, String);

        fn dynamic_list_structures(
            values: Vec<Box<DynamicAttributesStorage>>,
        ) -> Box<DynamicListStorage> {
            Box::new(DynamicListStorage(#dynamic::DynamicList::Struct(
                values
                    .into_iter()
                    .map(|value| {
                        let DynamicAttributesStorage(value) = *value;
                        #dynamic::DynamicStruct(value.into_vec())
                    })
                    .collect(),
            )))
        }

        fn dynamic_list_list(
            values: Vec<Box<DynamicListStorage>>,
        ) -> Box<DynamicListStorage> {
            Box::new(DynamicListStorage(#dynamic::DynamicList::List(
                values.into_iter().map(|value| value.0).collect(),
            )))
        }

        fn dynamic_attributes_vec_noop(_: &Vec<ffi::DynamicAttributes>) {}

        impl ffi::DynamicAttributes {
            pub fn into_model(mut self) -> #runtime::DynamicAttributes {
                assert_eq!(
                    self.storage.len(),
                    1,
                    "C++ dynamic attributes contained invalid storage",
                );
                self.storage.pop().expect("validated storage length").0
            }
        }
    };
    Ok(GeneratedFile {
        name: "dynamic_attributes.rs".to_owned(),
        content: pretty(tokens)?,
    })
}
