# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

import quent_tutorial_scoped_references as quent


def main() -> None:
    with quent.Context() as context:
        pipeline = context.pipeline_observer().handle()
        pipeline.created()

        task = context.task_observer().handle()
        task.started(parent=pipeline)
        task.ended()


if __name__ == "__main__":
    main()
