"use client";
import type { NamespaceStatistics } from "@/components/namespaces/table";
import type { QueueStatistics } from "@/components/queues/table";
import type { CreateNamespaceRequest } from "@/schemas/create-namespace";
import type { CreateQueueRequest } from "@/schemas/create-queue";
import type {
  QueueConfig,
  UpdateQueueConfigRequest,
} from "@/schemas/queue-settings";
import type { APIKey } from "@/components/create-api-key";
import type { UserStatistics } from "@/components/create-user";
import { SERVER_ENDPOINT } from "@/app/globals";
import type { CreateUserRequest } from "@/schemas/create-user";
import { toast } from "sonner";
import type { ApiKey } from "@/components/api-keys/table";
import type { Role } from "@/lib/state/global";
import type { MessageObject } from "@/app/(dashboard)/queues/list";

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
  return await fetch(`${SERVER_ENDPOINT}/stats/ns`, {
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

export async function listUserAllowedNamespaces({
  email,
}: {
  email?: string;
}): Promise<string[]> {
  if (email === undefined) {
    throw new Error("Email is required");
  }

  return await fetch(
    `${SERVER_ENDPOINT}/admin/users/${encodeURIComponent(email)}/permissions`,
    {
      method: "GET",
      credentials: "include",
      cache: "no-store",
      next: {
        tags: ["namespaces", "user-namespaces"],
      },
    },
  ).then((res) => res.json());
}

export async function updateUserAllowedNamespaces({
  email,
  namespaces,
}: {
  email: string;
  namespaces: string[];
}) {
  await fetch(
    `${SERVER_ENDPOINT}/admin/users/${encodeURIComponent(email)}/permissions`,
    {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(namespaces),
      next: {
        tags: ["namespaces", "user-namespaces"],
      },
    },
  ).catch(() => {
    toast.error("Something went wrong");
  });
}

export async function updateUserRole({
  email,
  role,
}: {
  email: string;
  role: Role;
}) {
  await fetch(
    `${SERVER_ENDPOINT}/admin/users/${encodeURIComponent(email)}/role`,
    {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ role }),
      next: {
        tags: ["users"],
      },
    },
  ).catch(() => toast.error("Something went wrong"));
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

export async function listQueues(): Promise<Map<string, QueueStatistics>> {
  return await fetch(`${SERVER_ENDPOINT}/stats/queue`, {
    method: "GET",
    credentials: "include",
    next: {
      tags: ["queues"],
    },
  })
    .then((res) => res.json())
    .then(
      (json: Record<string, QueueStatistics>) => new Map(Object.entries(json)),
    )
    .catch(() => {
      toast.error("Something went wrong");
      return new Map();
    });
}

export async function fetchQueue(
  namespace: string,
  queueName: string,
): Promise<QueueStatistics | undefined> {
  return await fetch(`${SERVER_ENDPOINT}/queue/${namespace}/${queueName}`, {
    method: "GET",
    credentials: "include",
    next: {
      tags: ["queues"],
    },
  }).then((res) => {
    if (res.status === 403) {
      throw new Error("Access Denied", {
        cause: 403,
      });
    }
    return res.json();
  });
}

export async function listMessages({
  queue,
  namespace,
}: {
  queue: string;
  namespace: string;
}): Promise<MessageObject[]> {
  return await fetch(
    `${SERVER_ENDPOINT}/queue/${namespace}/${queue}/messages`,
    {
      method: "GET",
      credentials: "include",
      next: {
        tags: ["queues", "queue-messages"],
      },
    },
  )
    .then((res) => res.json())
    .catch(() => {
      toast.error(
        `Something went wrong: failed to list messages for queue ${queue}`,
      );
      return [];
    });
}

export async function listAPIKeys(): Promise<ApiKey[]> {
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

export async function updateQueueSettings(data: UpdateQueueConfigRequest) {
  return await fetch(
    `${SERVER_ENDPOINT}/queue/${data.namespace}/${data.queue}/config`,
    {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        max_retries: data.maxRetries,
        dead_letter_queue: data.deadLetterQueue,
      }),
      next: {
        tags: ["queues", "queue-settings"],
      },
    },
  ).then((res) => {
    if (!res.ok) {
      throw new Error("Failed to update queue settings");
    }
  });
}

export async function getQueueSettings(
  namespace?: string,
  queue?: string,
): Promise<QueueConfig | undefined> {
  if (namespace === undefined || queue === undefined) {
    throw new Error("Invalid queue ID");
  }
  return await fetch(`${SERVER_ENDPOINT}/queue/${namespace}/${queue}/config`, {
    method: "GET",
    credentials: "include",
    cache: "no-store",
    next: {
      tags: ["queues", "queue-settings"],
    },
  })
    .then((res) => res.json())
    .then((data) => ({
      maxRetries: data.max_retries,
      deadLetterQueue: data.dead_letter_queue,
    }));
}
