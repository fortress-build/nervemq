"use client";

import { DataTable } from "@/components/data-table";
import type { ColumnDef, ColumnFiltersState } from "@tanstack/react-table";
import { ChevronDown, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useQuery } from "@tanstack/react-query";
import { listMessages } from "@/actions/api";
import { Filter, Check } from "lucide-react";
import { Popover, PopoverTrigger, PopoverContent } from "@/components/ui/popover";
import { Command, CommandInput, CommandItem, CommandList, CommandEmpty, CommandGroup } from "@/components/ui/command";
import React from "react";

export type MessageObject = {
  id: number;
  queue: string;
  body: string;
  tries: number;
  delivered_at: number;
  status: "pending" | "delivered" | "failed";

  message_attributes: Map<string, string | number>;
};

function MessageDetails({ message }: { message: MessageObject }) {
  return (
    <div className="p-6 space-y-4 bg-gray-50">
      <h3 className="font-semibold text-gray-700 mb-2">Message Details</h3>      
      {/* Message Body Section */}
      <div className="bg-white p-4 rounded-lg border border-gray-200">
        <span className="text-xs uppercase text-gray-400">Message Body</span>
        <div className="mt-1 text-sm text-gray-700 whitespace-pre-wrap">
          {message.body}
        </div>
      </div>

      {/* Existing Key-Value Pairs Section */}
      {Object.entries(message.kv).length === 0 ? (
      {Object.entries(message.message_attributes).length === 0 ? (
        <div className="bg-white p-4 rounded-lg border border-gray-200 text-gray-500 text-sm">
          No message details available
        </div>
      ) : (
        <div className="grid gap-3">
          {Object.entries(message.message_attributes)?.map(([k, v], index) => (
            <div
              key={`message-${index.toString()}`}
              className="bg-white p-4 rounded-lg border border-gray-200"
            >
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <span className="text-xs uppercase text-gray-400">Key</span>
                  <div className="mt-1 text-sm font-medium text-gray-700">
                    {k}
                  </div>
                </div>
                <div>
                  <span className="text-xs uppercase text-gray-400">Value</span>
                  <div className="mt-1 text-sm text-gray-700">{v}</div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// Define columns for the messages table
const columns: ColumnDef<MessageObject>[] = [
  {
    id: "expand",
    header: "",
    cell: ({ row }) => {
      return (
        <Button
          onClick={() => row.toggleExpanded()}
          className="p-2 hover:bg-gray-100 rounded bg-transparent w-10"
          variant="ghost"
        >
          {row.getIsExpanded() ? (
            <ChevronDown className="h-4 w-4" />
          ) : (
            <ChevronRight className="h-4 w-4" />
          )}
        </Button>
      );
    },
    enableResizing: false,
    enableHiding: false,
    size: 40,
    minSize: 40,
    maxSize: 40,
  },
  {
    accessorKey: "id",
    header: "ID",
  },
  {
    accessorKey: "status",
    header: ({ column }) => {
      const selectedStatus = column.getFilterValue() as string;
      const statuses = ["delivered", "failed", "pending"];

      return (
        <div className="flex items-center gap-2">
          <span>Status</span>
          <Popover>
            <PopoverTrigger asChild>
              <Button variant="ghost" className="p-0 hover:bg-transparent">
                <Filter className="h-4 w-4" />
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-[200px] p-0">
              <Command>
                <CommandInput placeholder="Filter status..." />
                <CommandList>
                  <CommandEmpty>No status found</CommandEmpty>
                  <CommandGroup>
                    {statuses.map((status) => (
                      <CommandItem
                        key={status}
                        value={status}
                        onSelect={(value) => {
                          column.setFilterValue(value === selectedStatus ? undefined : value);
                        }}
                      >
                        <Check
                          className={`mr-2 h-4 w-4 ${
                            selectedStatus === status ? "opacity-100" : "opacity-0"
                          }`}
                        />
                        {status}
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
  },
  {
    accessorKey: "tries",
    header: "Retries",
  },
];

export default function MessageList({
  queue,
  namespace,
}: {
  queue: string;
  namespace: string;
}) {
  const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>(
    []
  );
  
  const { data = [] } = useQuery({
    queryKey: ["queue-messages", { queue, namespace }],
    queryFn: () =>
      listMessages({
        queue,
        namespace,
      }),
  });

  return (
    <DataTable
      columns={columns}
      data={data}
      renderSubComponent={({ row }) => (
        <MessageDetails message={row.original} />
      )}
      columnFilters={columnFilters}
      setColumnFilters={setColumnFilters}
    />
  );
}
