/*
 * SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "quent-tutorial-finite-state-machine-cpp-bridge/gen/quent.hpp"

int main() {
  auto context = quent::Context::none();

  quent::FsmHandle<quent::Job> job = context.job_observer()->handle();

  // A transition consumes its move-only state handle and returns the handle
  // type for the new state. Named C++ objects require std::move to be consumed.
  quent::FsmHandle<quent::Job, quent::job_state::Queued> queued =
      std::move(job).queued();
  quent::FsmHandle<quent::Job, quent::job_state::LoadingInput> loading =
      std::move(queued).loading_input();
  quent::FsmHandle<quent::Job, quent::job_state::Running> running =
      std::move(loading).running();
  quent::FsmHandle<quent::Job, quent::job_state::Completed> completed =
      std::move(running).completed();
  return 0;
}
