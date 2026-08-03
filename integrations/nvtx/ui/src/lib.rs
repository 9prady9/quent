// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! TypeScript-facing projections of the reconstructed NVTX model.
//!
//! [`nvtx_analyzer`] is deliberately framework-free and owns no serialization or
//! binding concerns, so the types the UI consumes live here instead of on its
//! span type.
//!
//! Scaffold only: the lane model for rendering NVTX ranges alongside Quent's
//! executor threads lands on top of this.

/// The reconstructed model this crate projects.
///
/// Re-exported so a consumer of the UI types does not have to depend on the
/// reconstruction core directly.
pub use nvtx_analyzer::{NvtxModel, NvtxSpan};
