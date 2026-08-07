// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

import { describe, expect, it } from 'vitest';
import { stringifyJsonWithBigInt } from './stringifyJsonWithBigInt';

describe('stringifyJsonWithBigInt', () => {
  it('writes bigint values as unquoted lossless integer tokens', () => {
    expect(
      stringifyJsonWithBigInt({ domain_id: 18446744073709551615n, category_ids: [3, 7] })
    ).toBe('{"domain_id":18446744073709551615,"category_ids":[3,7]}');
  });

  it('preserves strings that happen to contain decimal digits', () => {
    expect(stringifyJsonWithBigInt({ query_key: '18446744073709551615' })).toBe(
      '{"query_key":"18446744073709551615"}'
    );
  });
});
