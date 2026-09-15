# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

import quent_tutorial_repeated_events as quent


def main() -> None:
    with quent.Context() as context:
        task = context.task_observer().handle()
        task.started(command="compile")
        task.progress(items_processed=64)
        task.progress(items_processed=128)
        task.ended(success=True)


if __name__ == "__main__":
    main()
