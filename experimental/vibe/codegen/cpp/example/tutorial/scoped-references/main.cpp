/*
 * SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "quent-tutorial-scoped-references-cpp-bridge/gen/quent.hpp"

int main() {
  auto context = quent::Context::none();
  auto pipeline = context.pipeline_observer()->handle();
  pipeline.created();

  auto task = context.task_observer()->handle();
  task.started(quent::task::Started{.parent = pipeline.id()});
  task.ended();
  return 0;
}
