"use client";
import type { CellContext, ColumnDef } from "@tanstack/react-table";
import { Trash2, KeySquare, ArrowUpDown, Braces } from "lucide-react";
import { Button } from "../ui/button";
import { useContext } from "react";
import { KeyToDeleteContext } from "@/lib/contexts/key-to-delete";

export type ApiKey = {
  name: string;
  namespace: string;
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
        <span>Name</span>
        <Button
          variant="ghost"
          className="p-0 hover:bg-transparent"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      </div>
    ),
    enableSorting: true,
  },
  {
    accessorKey: "namespace",
    header: ({ column }) => (
      <div className="flex items-center gap-2">
        <Braces className="h-4 w-4" />
        <span>Namespace</span>
        <Button
          variant="ghost"
          className="p-0 hover:bg-transparent"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      </div>
    ),
  },
  {
    id: "actions",
    cell: (row) => {
      return <ActionsCell context={row} />;
    },
  },
];
