// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

import type { NvtxViewportWindow, ZoomRange } from '@quent/utils';

export function nvtxViewportFromZoom(
  referenceUnixNs: bigint,
  zoomRange: ZoomRange
): NvtxViewportWindow {
  const startOffset = BigInt(Math.round(Math.max(0, zoomRange.start) * 1_000_000_000));
  const endOffset = BigInt(Math.round(Math.max(zoomRange.start, zoomRange.end) * 1_000_000_000));
  return {
    start: referenceUnixNs + startOffset,
    end: referenceUnixNs + endOffset,
  };
}
