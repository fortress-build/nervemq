"use client";
import type { ColumnDef } from "@tanstack/react-table";
import { DataTable } from "../data-table";

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

export function QueuesTable({ data }: { data: QueueStatistics[] }) {
  return (
    <div className="px-4 py-16 w-full">
      <DataTable columns={columns} data={data} />
    </div>
  );
}
