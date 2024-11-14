"use client";
import type { ColumnDef } from "@tanstack/react-table";
import { DataTable } from "../data-table";

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
    accessorKey: "ns",
    header: "Namespace",
  },
  {
    accessorKey: "messageCount",
    header: "Inflight",
  },
];

export function NamespaceTable({ data }: { data: NamespaceStatistics[] }) {
  return (
    <div className="px-4 py-16 w-full">
      <DataTable columns={columns} data={data} />
    </div>
  );
}
