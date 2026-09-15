# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

import quent_tutorial_minimal_model as quent


def main() -> None:
    with quent.Context() as context:
        task = context.task_observer().handle()
        task.started()
        task.ended()


if __name__ == "__main__":
    main()
