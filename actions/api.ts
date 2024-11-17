"use server";
import type { NamespaceStatistics } from "@/components/namespaces/table";
import type { QueueStatistics } from "@/components/queues/table";
import type { CreateNamespaceRequest } from "@/schemas/create-namespace";
import type { CreateQueueRequest } from "@/schemas/create-queue";
import type { APIKey } from "@/components/create-api-key";
import { revalidateTag } from "next/cache";
import type { UserStatistics } from "@/components/create-user";
import { SERVER_ENDPOINT } from "@/app/globals";

export async function createNamespace(data: CreateNamespaceRequest) {
  "use server";

  await fetch(`${SERVER_ENDPOINT}/ns/${data.name}`, {
    method: "POST",
    next: {
      tags: ["namespaces"],
    },
  });

  revalidateTag("namespaces");
}

export async function deleteNamespace(name: string) {
  "use server";

  await fetch(`${SERVER_ENDPOINT}/ns/${name}`, {
    method: "DELETE",
    next: {
      tags: ["namespaces"],
    },
  });

  revalidateTag("namespaces");
}

export async function listNamespaces(): Promise<NamespaceStatistics[]> {
  "use server";
  return await fetch(`${SERVER_ENDPOINT}/ns`, {
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

  await fetch(`${SERVER_ENDPOINT}/queue/${data.namespace}/${data.name}`, {
    method: "POST",
    next: {
      tags: ["queues"],
    },
  });

  revalidateTag("queues");
}

export async function deleteQueue(data: CreateQueueRequest) {
  "use server";

  await fetch(`${SERVER_ENDPOINT}/queue/${data.namespace}/${data.name}`, {
    method: "DELETE",
    next: {
      tags: ["queues"],
    },
  });

  revalidateTag("queues");
}

export async function listQueues(): Promise<QueueStatistics[]> {
  "use server";

  return await fetch(`${SERVER_ENDPOINT}/stats`, {
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

  return await fetch(`${SERVER_ENDPOINT}/api-keys`, {
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

  return await fetch(`${SERVER_ENDPOINT}/api-keys`, {
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

export async function deleteAPIKey(id: string) {
  "use server";

  await fetch(`${SERVER_ENDPOINT}/api-keys/${id}`, {
    method: "DELETE",
    next: {
      tags: ["api-keys"],
    },
  });

  revalidateTag("api-keys");
}

export async function createUser(username: string): Promise<UserStatistics> {
  "use server";

  return await fetch(`${SERVER_ENDPOINT}/users`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ username }),
    next: {
      tags: ["users"],
    },
  })
    .then((res) => res.json())
    .catch((e) => {
      console.error(e);
      throw e;
    });
}

export async function deleteUser(id: string) {
  "use server";

  await fetch(`${SERVER_ENDPOINT}/users/${id}`, {
    method: "DELETE",
    next: {
      tags: ["users"],
    },
  });

  revalidateTag("users");
}

export async function listUsers(): Promise<UserStatistics[]> {
  "use server";

  return await fetch(`${SERVER_ENDPOINT}/users`, {
    method: "GET",
    next: {
      tags: ["users"],
    },
  })
    .then((res) => res.json())
    .catch(() => []);
}
