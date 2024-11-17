"use client";

import { DataTable } from "@/components/data-table";
import { useQuery } from "@tanstack/react-query";
import type { ColumnDef } from "@tanstack/react-table";
import { useParams } from "next/navigation";

// Add these type definitions
type Message = {
  id: string;
  content: string;
  timestamp: string;
  status: "pending" | "completed" | "failed";
  error?: string; // Optional error message
  attempts?: number; // Number of processing attempts
};

// Mock data for development
const mockMessages: Message[] = [
  {
    id: "msg-1",
    content: "Process user registration",
    timestamp: "",
    status: "completed",
    attempts: 1,
  },
  {
    id: "msg-2",
    content: "Send welcome email",
    timestamp: "",
    status: "failed",
    error: "SMTP connection failed",
    attempts: 3,
  },
  {
    id: "msg-3",
    content: "Generate report",
    timestamp: "",
    status: "pending",
    attempts: 0,
  },
];

// Define columns for the messages table
const columns: ColumnDef<Message>[] = [
  {
    accessorKey: "id",
    header: "ID",
  },
  {
    accessorKey: "content",
    header: "Message",
  },
  {
    accessorKey: "timestamp",
    header: "Time",
    cell: ({ row }) => {
      return new Date(row.getValue("timestamp")).toLocaleString();
    },
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => {
      const status = row.getValue("status") as string;
      return (
        <span
          className={`px-2 py-1 rounded-full text-sm ${
            status === "completed"
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
    accessorKey: "attempts",
    header: "Attempts",
  },
  {
    accessorKey: "error",
    header: "Error",
    cell: ({ row }) => row.getValue("error") || "-",
  },
];

export default function ClientList() {
  const { queueId }: { queueId: string } = useParams();

  const { data = mockMessages } = useQuery({
    queryKey: ["queue-messages", queueId],
    queryFn: () => [] as Message[],
  });

  return <DataTable columns={columns} data={data} />;
}
