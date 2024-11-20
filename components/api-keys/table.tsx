"use client";
import type { CellContext, ColumnDef } from "@tanstack/react-table";
import { Trash2, KeySquare, ArrowUpDown } from "lucide-react";
import { Button } from "../ui/button";
import { useContext } from "react";
import { KeyToDeleteContext } from "@/lib/contexts/key-to-delete";

export type ApiKey = {
  name: string;
  createdAt: string;
};

function ActionsCell({
  context: { row },
}: {
  context: CellContext<ApiKey, unknown>;
}) {
  const cx = useContext(KeyToDeleteContext);

  return (
    <div className="flex items-center justify-end gap-2">
      <Button
        variant="ghost"
        size="sm"
        className="text-destructive hover:text-destructive hover:bg-destructive/10"
        onClick={() => {
          cx?.setKey(row.original.name);
        }}
      >
        <Trash2 className="h-4 w-4 text-destructive" />
      </Button>
    </div>
  );
}

export const columns: ColumnDef<ApiKey>[] = [
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
  // {
  //   accessorKey: "createdAt",
  //   header: () => (
  //     <div className="flex items-center gap-2">
  //       <Calendar className="h-4 w-4" />
  //       <span>Created</span>
  //     </div>
  //   ),
  //   cell: ({ row }) => new Date(row.original.createdAt).toLocaleDateString(),
  // },
  // {
  //   accessorKey: "lastUsed",
  //   header: () => (
  //     <div className="flex items-center gap-2">
  //       <Clock className="h-4 w-4" />
  //       <span>Last Used</span>
  //     </div>
  //   ),
  //   cell: ({ row }) =>
  //     row.original.lastUsed
  //       ? new Date(row.original.lastUsed).toLocaleDateString()
  //       : "Never",
  // },
  {
    id: "actions",
    cell: (row) => {
      return <ActionsCell context={row} />;
    },
  },
];
