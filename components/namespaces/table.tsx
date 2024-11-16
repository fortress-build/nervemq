"use client";
import type { ColumnDef } from "@tanstack/react-table";
import { ChevronRight, KeySquare, Logs, Trash2 } from "lucide-react";
import { ColumnHeader } from "../table-header";
import { Button } from "../ui/button";

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
    id: "chevron",
    cell: () => <ChevronRight className="h-4 w-4 text-muted-foreground" />,
  },
  {
    accessorKey: "name",
    header: () => <ColumnHeader label="Name" icon={KeySquare} />,
  },
  {
    accessorKey: "queueCount",
    header: () => <ColumnHeader label="Queues" icon={Logs} />,
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
