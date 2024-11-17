"use client";
import type { ColumnDef } from "@tanstack/react-table";
import {
  Activity,
  Braces,
  ChevronRight,
  KeySquare,
  Trash2,
} from "lucide-react";
import { ColumnHeader } from "../table-header";
import { Button } from "../ui/button";

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
    id: "chevron",
    cell: () => <ChevronRight className="h-4 w-4 text-muted-foreground" />,
  },
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
  {
    id: "actions",
    cell: (row) => (
      <div className="flex items-center justify-end gap-2">
        <Button
          variant="ghost"
          size="sm"
          className="text-destructive hover:text-destructive hover:bg-destructive/10"
          onClick={async (e) => {
            const meta = row.table.options.meta as
              | {
                  handleDeleteQueue: (
                    name: string,
                    ns: string,
                    e: unknown,
                  ) => void;
                }
              | undefined;
            meta?.handleDeleteQueue(
              row.row.original.name,
              row.row.original.ns,
              e,
            );
          }}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    ),
  },
];
