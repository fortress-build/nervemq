"use client";
import type { ColumnDef } from "@tanstack/react-table";
import {
  Braces,
  KeySquare,
  Trash2,
  ArrowUpDown,
  Filter,
  Check,
} from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import { Button } from "../ui/button";
import React from "react";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "../ui/command";
import { useQuery } from "@tanstack/react-query";
import { listNamespaces } from "@/lib/actions/api"; // adjust import path as needed

export type Queue = {
  id: string;
  ns: string;
  name: string;
};

export type QueueStatistics = Queue & {
  messageCount: number;
  avg_size_bytes: number;
  active_connections: number;
  pending: number;
  delivered: number;
  failed: number;
};

export const columns: ColumnDef<QueueStatistics>[] = [
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
    accessorKey: "ns",
    header: ({ column }) => {
      // eslint-disable-next-line react-hooks/rules-of-hooks
      const { data: namespaces = [] } = useQuery({
        queryFn: () => listNamespaces(),
        queryKey: ["namespaces"],
      });

      const selectedNamespaces = (column.getFilterValue() as string[]) || [];

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
            <PopoverContent className="w-[200px] p-0">
              <Command className="bg-background">
                <CommandInput placeholder="Search namespaces..." />
                <CommandList>
                  <CommandEmpty>No namespaces found</CommandEmpty>
                  <CommandGroup>
                    {namespaces.map((namespace) => (
                      <CommandItem
                        key={namespace.name}
                        value={namespace.name}
                        onSelect={(value) => {
                          const currentFilters =
                            (column.getFilterValue() as string[]) || [];
                          const newFilters = currentFilters.includes(value)
                            ? currentFilters.filter((f) => f !== value)
                            : [...currentFilters, value];
                          column.setFilterValue(
                            newFilters.length ? newFilters : undefined,
                          );
                        }}
                      >
                        <Check
                          className={`mr-2 h-4 w-4 ${
                            selectedNamespaces.includes(namespace.name)
                              ? "opacity-100"
                              : "opacity-0"
                          }`}
                        />
                        {namespace.name}
                      </CommandItem>
                    ))}
                  </CommandGroup>
                </CommandList>
              </Command>
            </PopoverContent>
          </Popover>
        </div>
      );
    },
    enableSorting: true,
    enableColumnFilter: true,
    filterFn: (row, columnId, filterValue: string[]) => {
      if (!filterValue?.length) return true;
      return filterValue.includes(row.getValue(columnId));
    },
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
