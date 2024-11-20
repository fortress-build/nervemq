"use client";
import type { ColumnDef } from "@tanstack/react-table";
import {
  Activity,
  Braces,
  KeySquare,
  Trash2,
  ArrowUpDown,
  Filter,
} from "lucide-react";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "../ui/popover";
import { Input } from "../ui/input";
import { ColumnHeader } from "../table-header";
import { Button } from "../ui/button";
import React from "react";

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
    header: () => (
      <div className="flex items-center gap-2">
        <KeySquare className="h-4 w-4" />
        <span>Name</span>
      </div>
    ),
  },
  {
    accessorKey: "ns",
    header: ({ column }) => {
      // eslint-disable-next-line react-hooks/rules-of-hooks
      const [filterValue, setFilterValue] = React.useState("");

      return (
        <div className="flex items-center gap-2">
          <Braces className="h-4 w-4" />
          <Button
            variant="ghost"
            className="p-0 hover:bg-transparent"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
          >
            <span>Namespace</span>
            <ArrowUpDown className="ml-2 h-4 w-4" />
          </Button>
          <Popover>
            <PopoverTrigger asChild>
              <Button variant="ghost" className="p-0 hover:bg-transparent">
                <Filter className="h-4 w-4" />
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-60">
              <div className="space-y-2">
                <h4 className="font-medium leading-none">Filter Namespace</h4>
                <Input
                  placeholder="Search namespaces..."
                  value={filterValue}
                  onChange={(event) => {
                    const value = event.target.value;
                    setFilterValue(value);
                    column.setFilterValue(value);
                  }}
                  className="h-8"
                />
              </div>
            </PopoverContent>
          </Popover>
        </div>
      );
    },
    enableSorting: true,
    enableColumnFilter: true,
    filterFn: "includesString",
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
