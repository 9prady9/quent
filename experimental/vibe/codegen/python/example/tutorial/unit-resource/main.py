# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

import quent_tutorial_unit_resource as quent


def main() -> None:
    with quent.Context() as context:
        pool = context.thread_pool_observer().handle()
        pool.created()

        thread = context.thread_observer().handle()
        thread.registered(pool=pool)

        task = context.task_observer().handle()
        running = task.running(thread={"target": thread, "data": {}})
        completed = running.completed()


if __name__ == "__main__":
    main()
