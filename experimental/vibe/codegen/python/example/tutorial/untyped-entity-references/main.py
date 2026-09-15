# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

import quent_tutorial_untyped_entity_references as quent


def main() -> None:
    with quent.Context() as context:
        worker = context.worker_observer().handle()
        worker.started()

        task = context.task_observer().handle()
        task.started(source=worker.uuid)
        task.ended()
        worker.ended()


if __name__ == "__main__":
    main()
