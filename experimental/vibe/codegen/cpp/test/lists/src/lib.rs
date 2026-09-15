// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused, clippy::all)]
mod instrumentation {
    include!(concat!(env!("OUT_DIR"), "/instrumentation.rs"));
}

#[allow(unused, clippy::all)]
mod bridge {
    include!(concat!(env!("OUT_DIR"), "/bridge_mod.rs"));
}

#[cfg(test)]
mod tests {
    unsafe extern "C" {
        fn quent_cpp_list_smoke() -> std::ffi::c_int;
    }

    #[test]
    fn compiles_and_converts_all_list_shapes() {
        assert_eq!(unsafe { quent_cpp_list_smoke() }, 0);
    }
}
