"use client";
import type { NamespaceStatistics } from "@/components/namespaces/table";
import type { QueueStatistics } from "@/components/queues/table";
import type { CreateNamespaceRequest } from "@/schemas/create-namespace";
import type { CreateQueueRequest } from "@/schemas/create-queue";
import type { APIKey } from "@/components/create-api-key";
import type { UserStatistics } from "@/components/create-user";
import { SERVER_ENDPOINT } from "@/app/globals";
import type { CreateUserRequest } from "@/schemas/create-user";

export async function createNamespace(data: CreateNamespaceRequest) {
  await fetch(`${SERVER_ENDPOINT}/ns/${data.name}`, {
    method: "POST",
    credentials: "include",
    next: {
      tags: ["namespaces"],
    },
  });
}

export async function deleteNamespace(name: string) {
  await fetch(`${SERVER_ENDPOINT}/ns/${name}`, {
    method: "DELETE",
    credentials: "include",
    next: {
      tags: ["namespaces"],
    },
  });
}

export async function listNamespaces(): Promise<NamespaceStatistics[]> {
  return await fetch(`${SERVER_ENDPOINT}/ns`, {
    method: "GET",
    credentials: "include",
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
  await fetch(`${SERVER_ENDPOINT}/queue/${data.namespace}/${data.name}`, {
    method: "POST",
    credentials: "include",
    next: {
      tags: ["queues"],
    },
  });
}

export async function deleteQueue(data: CreateQueueRequest) {
  await fetch(`${SERVER_ENDPOINT}/queue/${data.namespace}/${data.name}`, {
    method: "DELETE",
    credentials: "include",
    next: {
      tags: ["queues"],
    },
  });
}

export async function listQueues(): Promise<QueueStatistics[]> {
  return await fetch(`${SERVER_ENDPOINT}/stats`, {
    method: "GET",
    credentials: "include",
    next: {
      tags: ["queues"],
    },
  })
    .then((res) => res.json())
    .catch(() => []);
}

export async function listAPIKeys(): Promise<APIKey[]> {
  "use client";
  // window.Headers
  // console.log(await cookies())
  return await fetch(`${SERVER_ENDPOINT}/tokens`, {
    method: "GET",
    credentials: "include",
    mode: "cors",
    next: {
      tags: ["api-keys"],
    },
  })
    .then((res) => {
      console.log(res);
      return res.json();
    })
    .catch(() => []);
}

export async function createAPIKey(name: string): Promise<APIKey> {
  return await fetch(`${SERVER_ENDPOINT}/tokens`, {
    method: "POST",
    credentials: "include",
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
  await fetch(`${SERVER_ENDPOINT}/tokens/${id}`, {
    method: "DELETE",
    credentials: "include",
    next: {
      tags: ["api-keys"],
    },
  });
}

export async function createUser(data: CreateUserRequest): Promise<void> {
  await fetch(`${SERVER_ENDPOINT}/admin/users`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
    next: {
      tags: ["users"],
    },
  });
}

export type DeleteUserRequest = {
  email: string;
};

export async function deleteUser(data: DeleteUserRequest) {
  await fetch(`${SERVER_ENDPOINT}/admin/users`, {
    method: "DELETE",
    credentials: "include",
    body: JSON.stringify(data),
    next: {
      tags: ["users"],
    },
  });
}

export async function listUsers(): Promise<UserStatistics[]> {
  return await fetch(`${SERVER_ENDPOINT}/admin/users`, {
    method: "GET",
    credentials: "include",
    next: {
      tags: ["users"],
    },
  }).then((res) => res.json());
}
