// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

import { describe, expect, it } from 'vitest';
import { nvtxViewportFromZoom } from './nvtxTimeline.utils';

describe('nvtxViewportFromZoom', () => {
  it('adds relative seconds to the Unix reference without Number precision loss', () => {
    const reference = 1_800_000_000_000_000_001n;
    expect(nvtxViewportFromZoom(reference, { start: 0.25, end: 1.5 })).toEqual({
      start: 1_800_000_000_250_000_001n,
      end: 1_800_000_001_500_000_001n,
    });
  });
});
