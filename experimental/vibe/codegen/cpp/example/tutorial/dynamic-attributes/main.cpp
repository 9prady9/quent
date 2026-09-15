/*
 * SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "quent-tutorial-dynamic-attributes-cpp-bridge/gen/quent.hpp"

int main() {
  auto context = quent::Context::none();
  auto task = context.task_observer()->handle();

  quent::DynamicAttributes started_details;
  started_details.add("queue", "priority");
  started_details.add("attempt", std::uint64_t{2});
  task.started(quent::task::Started{.details = std::move(started_details)});

  quent::DynamicAttributes ended_details;
  ended_details.add("cached", false);
  ended_details.add("items_processed", std::uint64_t{128});
  task.ended(quent::task::Ended{.details = std::move(ended_details)});
  return 0;
}
