"use client";
import type { ColumnDef } from "@tanstack/react-table";
import { Activity, Braces, KeySquare } from "lucide-react";
import { ColumnHeader } from "../table-header";

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
    header: () => <ColumnHeader label="Name" icon={KeySquare} />,
  },
  {
    accessorKey: "ns",
    header: () => <ColumnHeader label="Namespace" icon={Braces} />,
  },
  {
    accessorKey: "messageCount",
    header: () => <ColumnHeader label="Inflight" icon={Activity} />,
  },
];
