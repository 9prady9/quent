# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

import quent_tutorial_fsm_self_loop as quent


def main() -> None:
    with quent.Context() as context:
        task = context.task_observer().handle()
        running = task.running(items_processed=0)
        running = running.running(items_processed=64)
        paused = running.paused()
        running = paused.running(items_processed=128)
        completed = running.completed()


if __name__ == "__main__":
    main()
