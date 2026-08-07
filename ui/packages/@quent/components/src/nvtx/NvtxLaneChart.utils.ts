// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

/** Subtract the Unix reference while values are still bigint, then convert. */
export function nvtxUnixNsToRelativeMs(timestamp: bigint, referenceUnixNs: bigint): number {
  const delta = timestamp - referenceUnixNs;
  return Number(delta) / 1_000_000;
}

/** Format an unsigned nanosecond duration without converting through Number. */
export function formatNvtxDurationNs(duration: bigint): string {
  if (duration < 1_000n) return `${duration.toString()} ns`;
  return `${formatMilliseconds(duration, duration < 1_000_000n ? 3 : 2)} ms`;
}

function formatMilliseconds(duration: bigint, decimalPlaces: number): string {
  const decimalScale = 10n ** BigInt(decimalPlaces);
  const rounded = (duration * decimalScale + 500_000n) / 1_000_000n;
  const whole = rounded / decimalScale;
  const fraction = (rounded % decimalScale).toString().padStart(decimalPlaces, '0');
  return `${whole.toString()}.${fraction}`;
}
