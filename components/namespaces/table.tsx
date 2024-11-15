"use client";
import type { ColumnDef } from "@tanstack/react-table";

export type Namespace = {
  id: string;
  ns: string;
  name: string;
};

export type NamespaceStatistics = Namespace & {
  messageCount: number;
};

export const columns: ColumnDef<NamespaceStatistics>[] = [
  {
    accessorKey: "name",
    header: "Name",
  },
  {
    accessorKey: "queueCount",
    header: "Queues",
  },
];
