/*
 * SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "quent-tutorial-event-data-cpp-bridge/gen/quent.hpp"

int main() {
  auto context = quent::Context::none();
  auto task = context.task_observer()->handle();
  task.started(quent::task::Started{
      .enabled = true,
      .byte = 1,
      .short_count = 2,
      .attempt = 3,
      .item_count = 4,
      .small_offset = -1,
      .short_offset = -2,
      .offset = -3,
      .large_offset = -4,
  });

  quent::DynamicAttributes extra;
  extra.add("worker", "alpha");
  extra.add("queue_depth", std::uint64_t{3});
  task.ended(quent::task::Ended{
      .ratio = 0.5F,
      .score = 0.95,
      .message = "complete",
      .run_id = quent::now_v7(),
      .retry_after = std::nullopt,
      .tags = {"batch", "priority"},
      .extra = std::move(extra),
  });
  return 0;
}
