"use server";

import type { CreateQueue } from "@/components/create-queue";
import type { QueueStatistics } from "@/components/queues/table";

export async function createQueue(data: CreateQueue) {
  "use server";

  console.log(data);

  await new Promise((resolve) => {
    setTimeout(() => {
      resolve(null);
    }, 2000);
  });
}

export async function listQueues(): Promise<QueueStatistics[]> {
  "use server";

  const res = await fetch("http://localhost:8080/stats/", {
    method: "GET",
  })
    .then((res) => res.json())
    .catch(() => ({
      queues: [],
    }));

  return res.queues;
}
