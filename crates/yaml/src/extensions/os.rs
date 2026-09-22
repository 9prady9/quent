// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Operating-system identity extension.

use quent_os::{process_path, process_record, thread_path, thread_record};
use quent_schema::{DataType, Entity, Field, Record};
use serde::Deserialize;

/// An operating-system identity type.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OsType {
    pub(crate) os: OsRole,
}

/// The operating-system identity represented by a field.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OsRole {
    Process,
    Thread,
}

/// Elaborate an operating-system identity type into its canonical record reference.
pub(super) fn elaborate_type(os: &OsType) -> DataType {
    DataType::Record(match os.os {
        OsRole::Process => process_path(),
        OsRole::Thread => thread_path(),
    })
}

/// Return canonical OS records referenced by the lowered schema but not declared.
pub(super) fn referenced_records(records: &[Record], entities: &[Entity]) -> Vec<Record> {
    let mut process = false;
    let mut thread = false;
    for ty in records
        .iter()
        .flat_map(|record| record.fields().map(Field::ty))
        .chain(entities.iter().flat_map(|entity| {
            entity
                .events()
                .flat_map(|event| event.fields().map(Field::ty))
        }))
    {
        find_references(ty, &mut process, &mut thread);
    }

    let mut generated = Vec::new();
    if process
        && records
            .iter()
            .all(|record| record.path() != &process_path())
    {
        generated.push(process_record());
    }
    if thread && records.iter().all(|record| record.path() != &thread_path()) {
        generated.push(thread_record());
    }
    generated
}

fn find_references(ty: &DataType, process: &mut bool, thread: &mut bool) {
    match ty {
        DataType::Record(path) if path == &process_path() => *process = true,
        DataType::Record(path) if path == &thread_path() => *thread = true,
        DataType::Option(inner) | DataType::List(inner) => {
            find_references(inner, process, thread);
        }
        DataType::EntityRef {
            data: Some(data), ..
        } => find_references(data, process, thread),
        _ => {}
    }
}
