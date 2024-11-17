"use client";
import type { ColumnDef } from "@tanstack/react-table";
import { Trash2, KeySquare, Calendar, Clock } from "lucide-react";
import { Button } from "../ui/button";

export type ApiKey = {
  id: string;
  name: string;
  createdAt: string;
  lastUsed?: string;
};

export const columns: ColumnDef<ApiKey>[] = [
  {
    accessorKey: "name",
    header: () => (
      <div className="flex items-center gap-2">
        <KeySquare className="h-4 w-4" />
        <span>Name</span>
      </div>
    ),
  },
  {
    accessorKey: "createdAt",
    header: () => (
      <div className="flex items-center gap-2">
        <Calendar className="h-4 w-4" />
        <span>Created</span>
      </div>
    ),
    cell: ({ row }) => new Date(row.original.createdAt).toLocaleDateString(),
  },
  {
    accessorKey: "lastUsed",
    header: () => (
      <div className="flex items-center gap-2">
        <Clock className="h-4 w-4" />
        <span>Last Used</span>
      </div>
    ),
    cell: ({ row }) =>
      row.original.lastUsed
        ? new Date(row.original.lastUsed).toLocaleDateString()
        : "Never",
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
                  handleDeleteKey: (id: string, e: unknown) => void;
                }
              | undefined;
            meta?.handleDeleteKey(row.row.original.id, e);
          }}
        >
          <Trash2 className="h-4 w-4 text-destructive" />
        </Button>
      </div>
    ),
  },
];

