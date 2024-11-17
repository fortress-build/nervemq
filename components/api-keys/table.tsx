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
    cell: ({ row, table }) => {
      const meta = table.options.meta as {
        handleDeleteKey: (id: string) => void;
      };

      return (
        <Button
          variant="ghost"
          size="icon"
          onClick={(e) => {
            e.stopPropagation();
            meta.handleDeleteKey(row.original.id);
          }}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      );
    },
  },
];

