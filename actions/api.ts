"use server";

import type { CreateQueue } from "@/components/create-queue";
import type { NamespaceStatistics } from "@/components/namespaces/table";
import type { QueueStatistics } from "@/components/queues/table";
import { revalidateTag } from "next/cache";

export async function createNamespace(name: string) {
  "use server";

  await fetch(`http://localhost:8080/ns/${name}`, {
    method: "POST",
    next: {
      tags: ["namespaces"],
    },
  });

  revalidateTag("namespaces");
}

export async function deleteNamespace(name: string) {
  "use server";

  await fetch(`http://localhost:8080/ns/${name}`, {
    method: "DELETE",
    next: {
      tags: ["namespaces"],
    },
  });

  revalidateTag("namespaces");
}

export async function listNamespaces(): Promise<NamespaceStatistics[]> {
  "use server";
  return await fetch("http://localhost:8080/ns", {
    method: "GET",
    next: {
      tags: ["namespaces"],
    },
  })
    .then((res) => res.json())
    .catch((e) => {
      console.error(e);

      return [];
    });
}

export async function createQueue(data: CreateQueue) {
  "use server";

  await fetch(`http://localhost:8080/queue/${data.namespace}/${data.name}`, {
    method: "POST",
    next: {
      tags: ["queues"],
    },
  });

  revalidateTag("queues");
}

export async function deleteQueue(data: CreateQueue) {
  "use server";

  await fetch(`http://localhost:8080/queue/default/${data.name}`, {
    method: "DELETE",
    next: {
      tags: ["queues"],
    },
  });

  revalidateTag("queues");
}

export async function listQueues(): Promise<QueueStatistics[]> {
  "use server";

  return await fetch("http://localhost:8080/stats", {
    method: "GET",
    next: {
      tags: ["queues"],
    },
  })
    .then((res) => res.json())
    .catch(() => []);
}
