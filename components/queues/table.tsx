"use client";
import type { ColumnDef } from "@tanstack/react-table";

export type Queue = {
  id: string;
  ns: string;
  name: string;
};

export type QueueStatistics = Queue & {
  messageCount: number;
};

export const columns: ColumnDef<QueueStatistics>[] = [
  {
    accessorKey: "name",
    header: "Name",
  },
  {
    accessorKey: "ns",
    header: "Namespace",
  },
  {
    accessorKey: "messageCount",
    header: "Inflight",
  },
];
