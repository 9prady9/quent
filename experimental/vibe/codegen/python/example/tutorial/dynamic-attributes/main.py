# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

import quent_tutorial_dynamic_attributes as quent


def main() -> None:
    with quent.Context() as context:
        task = context.task_observer().handle()
        task.started(
            details={"queue": "priority", "attempt": quent.DynamicValue.u64(2)}
        )
        task.ended(
            details={
                "cached": False,
                "items_processed": quent.DynamicValue.u64(128),
            }
        )


if __name__ == "__main__":
    main()
