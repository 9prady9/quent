/*
 * SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "quent-tutorial-repeated-events-cpp-bridge/gen/quent.hpp"

int main() {
  auto context = quent::Context::none();
  auto task = context.task_observer()->handle();
  task.started(quent::task::Started{.command = "compile"});
  task.progress(quent::task::Progress{.items_processed = 64});
  task.progress(quent::task::Progress{.items_processed = 128});
  task.ended(quent::task::Ended{.success = true});
  return 0;
}
