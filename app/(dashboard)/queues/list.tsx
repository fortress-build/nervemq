"use client";

import { DataTable } from "@/components/data-table";
import type { ColumnDef } from "@tanstack/react-table";
import { ChevronDown, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useQuery } from "@tanstack/react-query";
import { listMessages } from "@/actions/api";

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
          className="p-2 hover:bg-gray-100 rounded bg-transparent"
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
    size: 40,
  },
  {
    accessorKey: "id",
    header: "ID",
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => {
      const delivered = row.original.delivered_at > 0;
      const status = delivered
        ? "delivered"
        : row.original.tries > 3
          ? "failed"
          : "pending";
      return (
        <span
          className={`px-2 py-1 rounded-full text-sm ${
            status === "delivered"
              ? "bg-green-100 text-green-800"
              : status === "failed"
                ? "bg-red-100 text-red-800"
                : "bg-yellow-100 text-yellow-800"
          }`}
        >
          {status}
        </span>
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
    />
  );
}
