"use client";
import type { ColumnDef } from "@tanstack/react-table";
import { KeySquare, Logs } from "lucide-react";
import { ColumnHeader } from "../table-header";

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
    header: () => <ColumnHeader label="Name" icon={KeySquare} />,
  },
  {
    accessorKey: "queueCount",
    header: () => <ColumnHeader label="Queues" icon={Logs} />,
  },
];
