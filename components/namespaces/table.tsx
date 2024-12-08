"use client";
import type { ColumnDef } from "@tanstack/react-table";
import { KeySquare, Logs, Trash2, ArrowUpDown } from "lucide-react";
import { Button } from "../ui/button";

export type Namespace = {
  id: string;
  ns: string;
  name: string;
};

export type NamespaceStatistics = Namespace & {
  queueCount: number;
};

export const columns: ColumnDef<NamespaceStatistics>[] = [
  {
    accessorKey: "name",
    header: ({ column }) => (
      <div className="flex items-center gap-2">
        <KeySquare className="h-4 w-4" />
        <Button
          variant="ghost"
          className="p-0 hover:bg-transparent"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          <span>Name</span>
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      </div>
    ),
    enableSorting: true,
  },
  {
    accessorKey: "queueCount",
    header: () => (
      <div className="flex items-center gap-2">
        <Logs className="h-4 w-4" />
        <span>Queues</span>
      </div>
    ),
  },
  {
    id: "actions",
    cell: (row) => (
      <div className="flex items-center justify-end gap-2">
        <Button
          variant="ghost"
          size="sm"
          className="text-destructive hover:text-destructive hover:bg-destructive/10"
          onClick={(e) => {
            const meta = row.table.options.meta as
              | {
                  handleDeleteNamespace: (name: string, e: unknown) => void;
                }
              | undefined;
            meta?.handleDeleteNamespace(row.row.original.name, e);
          }}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    ),
  },
];
