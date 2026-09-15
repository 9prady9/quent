# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

import quent_tutorial_event_data as quent


def main() -> None:
    with quent.Context() as context:
        task = context.task_observer().handle()
        task.started(
            enabled=True,
            byte=1,
            short_count=2,
            attempt=3,
            item_count=4,
            small_offset=-1,
            short_offset=-2,
            offset=-3,
            large_offset=-4,
        )
        task.ended(
            ratio=0.5,
            score=0.95,
            message="complete",
            run_id=quent.now_v7(),
            retry_after=None,
            tags=["batch", "priority"],
            extra={
                "worker": "alpha",
                "queue_depth": quent.DynamicValue.u64(3),
            },
        )


if __name__ == "__main__":
    main()
