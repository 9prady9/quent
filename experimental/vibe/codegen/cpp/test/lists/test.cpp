/*
 * SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "quent-cpp-list-test/gen/quent.hpp"

extern "C" int quent_cpp_list_smoke() {
  auto context = quent::Context::none();
  auto batch = context.batch_observer()->handle();

  std::vector<quent::DynamicAttributes> extras;
  quent::DynamicAttributes first;
  first.add("name", "first");
  extras.push_back(std::move(first));
  quent::DynamicAttributes second;
  second.add("value", std::uint32_t{2});
  extras.push_back(std::move(second));

  batch.recorded(quent::batch::Recorded{
      .flags = {true, false},
      .ids = {quent::now_v7(), quent::now_v7()},
      .items = {{.value = 1}, {.value = 2}},
      .extras = std::move(extras),
      .matrix = {{1, 2}, {3, 4}},
  });
  return 0;
}
