// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

import { stringify } from 'json-custom-numbers';

/** Serialize BigInt values as lossless, unquoted JSON integer tokens. */
export function stringifyJsonWithBigInt(value: unknown): string {
  return stringify(value, undefined, undefined, (_key, item, itemType) =>
    itemType === 'bigint' ? (item as bigint).toString(10) : undefined
  );
}
