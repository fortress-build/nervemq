"use server";
import type { NamespaceStatistics } from "@/components/namespaces/table";
import type { QueueStatistics } from "@/components/queues/table";
import type { CreateNamespaceRequest } from "@/schemas/create-namespace";
import type { CreateQueueRequest } from "@/schemas/create-queue";
import type { APIKey } from "@/components/create-api-key";
import { revalidateTag } from "next/cache";

export async function createNamespace(data: CreateNamespaceRequest) {
  "use server";

  await fetch(`http://localhost:8080/ns/${data.name}`, {
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

export async function createQueue(data: CreateQueueRequest) {
  "use server";

  await fetch(`http://localhost:8080/queue/${data.namespace}/${data.name}`, {
    method: "POST",
    next: {
      tags: ["queues"],
    },
  });

  revalidateTag("queues");
}

export async function deleteQueue(data: CreateQueueRequest) {
  "use server";

  await fetch(`http://localhost:8080/queue/${data.namespace}/${data.name}`, {
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

export async function listAPIKeys(): Promise<APIKey[]> {
  "use server";

  return await fetch("http://localhost:8080/api-keys", {
    method: "GET",
    next: {
      tags: ["api-keys"],
    },
  })
    .then((res) => res.json())
    .catch(() => []);
}

export async function createAPIKey(name: string): Promise<APIKey> {
  "use server";

  return await fetch("http://localhost:8080/api-keys", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ name }),
    next: {
      tags: ["api-keys"],
    },
  })
    .then((res) => res.json())
    .catch((e) => {
      console.error(e);
      throw e;
    });
}

