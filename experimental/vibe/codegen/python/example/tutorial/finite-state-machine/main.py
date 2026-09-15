# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

import quent_tutorial_finite_state_machine as quent


def main() -> None:
    with quent.Context() as context:
        completed = (
            context.job_observer()
            .handle()
            .queued()
            .loading_input()
            .running()
            .completed()
        )


if __name__ == "__main__":
    main()
