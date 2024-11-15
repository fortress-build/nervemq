"use client";

import { useQuery } from "@tanstack/react-query";
import { DataTable } from "@/components/data-table";
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
const columns = [
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


const QueuePage = () => {
  const { queueId }: { queueId: string } = useParams();

  const { data = mockMessages } = useQuery({
    queryKey: ["queue-messages", queueId],
    // queryFn: () => fetchQueueMessages(queueId),
  });

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-medium mb-4">Queue #{queueId}</h1>

      <div className="grid gap-4">
        {/* Queue Status Section */}
        <section className="border rounded-lg p-4">
          <h2 className="text-xl font-medium mb-2">Queue Status</h2>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <p className="text-gray-600">Pending</p>
              <p className="text-2xl font-medium">5</p>
            </div>
            <div>
              <p className="text-gray-600">Processing</p>
              <p className="text-2xl font-medium">5</p>
            </div>
            <div>
              <p className="text-gray-600">Completed</p>
              <p className="text-2xl font-medium">5</p>
            </div>
            <div>
              <p className="text-gray-600">Failed</p>
              <p className="text-2xl font-medium">5</p>
            </div>
          </div>
        </section>

        {/* Current Queue Items */}
        <section className="border rounded-lg p-4">
          <h2 className="text-xl font-medium mb-2">Current Items</h2>
          <DataTable columns={columns} data={data} />
        </section>

        {/* Queue Controls */}
        <section className="border rounded-lg p-4">
          <h2 className="text-xl font-medium mb-2">Controls</h2>
          {/* Add queue control buttons */}
        </section>
      </div>
    </div>
  );
};

export default QueuePage;
