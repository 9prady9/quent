/*
 * SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "quent-tutorial-minimal-model-cpp-bridge/gen/quent.hpp"

int main() {
  auto context = quent::Context::none();
  quent::Handle<quent::Task> task = context.task_observer()->handle();
  task.started();
  task.ended();
  return 0;
}
