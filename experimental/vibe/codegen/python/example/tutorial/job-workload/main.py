# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

import quent_tutorial_job_workload as quent


def main() -> None:
    with quent.Context() as context:
        worker = context.worker_observer().handle()
        worker.ready(name="worker-1", limits={"threads": 16})

        job = context.job_observer().handle()
        queued = job.queued(name="compile", requested_threads=4)
        running = queued.running(
            worker={
                "target": worker,
                "data": {"threads": 4},
            }
        )
        completed = running.completed()


if __name__ == "__main__":
    main()
