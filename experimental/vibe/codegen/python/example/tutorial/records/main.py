# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

import quent_tutorial_records as quent


def main() -> None:
    with quent.Context() as context:
        task = context.task_observer().handle()
        batch = context.batch_observer().handle()

        task.started()
        task.ended(result={"success": True, "items_processed": 128})

        batch.started()
        batch.ended(result={"success": True, "items_processed": 512})


if __name__ == "__main__":
    main()
