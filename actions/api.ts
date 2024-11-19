"use client";
import type { NamespaceStatistics } from "@/components/namespaces/table";
import type { QueueStatistics } from "@/components/queues/table";
import type { CreateNamespaceRequest } from "@/schemas/create-namespace";
import type { CreateQueueRequest } from "@/schemas/create-queue";
import type { APIKey } from "@/components/create-api-key";
import type { UserStatistics } from "@/components/create-user";
import { SERVER_ENDPOINT } from "@/app/globals";
import type { CreateUserRequest } from "@/schemas/create-user";
import { toast } from "sonner";

export async function createNamespace(data: CreateNamespaceRequest) {
  await fetch(`${SERVER_ENDPOINT}/ns/${data.name}`, {
    method: "POST",
    credentials: "include",
    next: {
      tags: ["namespaces"],
    },
  }).catch(() => toast.error("Something went wrong"));
}

export async function deleteNamespace(name: string) {
  await fetch(`${SERVER_ENDPOINT}/ns/${name}`, {
    method: "DELETE",
    credentials: "include",
    next: {
      tags: ["namespaces"],
    },
  }).catch(() => toast.error("Something went wrong"));
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
    .catch(() => {
      toast.error("Something went wrong");

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
  }).catch(() => toast.error("Something went wrong"));
}

export async function deleteQueue(data: CreateQueueRequest) {
  await fetch(`${SERVER_ENDPOINT}/queue/${data.namespace}/${data.name}`, {
    method: "DELETE",
    credentials: "include",
    next: {
      tags: ["queues"],
    },
  }).catch(() => toast.error("Something went wrong"));
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
    .catch(() => {
      toast.error("Something went wrong");
      return [];
    });
}

export async function listAPIKeys(): Promise<APIKey[]> {
  "use client";
  return await fetch(`${SERVER_ENDPOINT}/tokens`, {
    method: "GET",
    credentials: "include",
    mode: "cors",
    next: {
      tags: ["api-keys"],
    },
  })
    .then((res) => res.json())
    .catch(() => {
      toast.error("Something went wrong");
      return [];
    });
}

export type CreateTokenRequest = {
  name: string;
};

export async function createAPIKey(req: CreateTokenRequest): Promise<APIKey> {
  return await fetch(`${SERVER_ENDPOINT}/tokens`, {
    method: "POST",
    credentials: "include",
    body: JSON.stringify(req),
    next: {
      tags: ["api-keys"],
    },
  })
    .then((res) => res.json())
    .catch(() => {
      toast.error("Something went wrong");
    });
}

export type DeleteTokenRequest = {
  name: string;
};

export async function deleteAPIKey(req: DeleteTokenRequest) {
  await fetch(`${SERVER_ENDPOINT}/tokens`, {
    method: "DELETE",
    body: JSON.stringify(req),
    credentials: "include",
    next: {
      tags: ["api-keys"],
    },
  }).catch(() => toast.error("Something went wrong"));
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
  }).catch(() => toast.error("Something went wrong"));
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
  }).catch(() => toast.error("Something went wrong"));
}

export async function listUsers(): Promise<UserStatistics[]> {
  return await fetch(`${SERVER_ENDPOINT}/admin/users`, {
    method: "GET",
    credentials: "include",
    next: {
      tags: ["users"],
    },
  })
    .then((res) => res.json())
    .catch(() => toast.error("Something went wrong"));
}
