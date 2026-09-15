/*
 * SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "quent-tutorial-untyped-entity-references-cpp-bridge/gen/quent.hpp"

int main() {
  auto context = quent::Context::none();
  auto worker = context.worker_observer()->handle();
  worker.started();

  auto task = context.task_observer()->handle();
  task.started(quent::task::Started{.source = worker.id().raw()});
  task.ended();
  worker.ended();
  return 0;
}
