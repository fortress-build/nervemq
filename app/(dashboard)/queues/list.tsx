"use client";

import { DataTable } from "@/components/data-table";
import type { ColumnDef } from "@tanstack/react-table";
import { ChevronDown, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";

// Add these type definitions
type Message = {
  key: string;
  value: string;
};

type EventObject = {
  id: string;
  content: string;
  timestamp: string;
  status: "pending" | "completed" | "failed";
  error?: string;
  attempts?: number;
  messages?: Message[];
};

// Mock data for development
const mockEvents: EventObject[] = [
  {
    id: "msg-1234-abcd",
    content: "Process user registration for john@example.com",
    timestamp: "2024-03-20T10:30:00Z",
    status: "completed",
    attempts: 1,
    messages: [
      {
        key: "Registration Start",
        value: "Started processing registration"
      },
      {
        key: "Registration Complete",
        value: "Registration completed successfully"
      }
    ]
  },
  {
    id: "msg-5678-efgh",
    content: "Generate monthly sales report - March 2024",
    timestamp: "2024-03-20T10:29:30Z",
    status: "pending",
    attempts: 1,
    messages: [
      {
        key: "Sales Report Start",
        value: "Started generating sales report"
      }
    ]
  },
  {
    id: "msg-90ab-ijkl",
    content: "Send password reset email to sarah@example.com",
    timestamp: "2024-03-20T10:28:45Z",
    status: "failed",
    error: "User not found in database",
    attempts: 3,
    messages: [
      {
        key: "Password Reset Start",
        value: "Started sending password reset email"
      },
      {
        key: "User Not Found",
        value: "User not found in database"
      }
    ]
  },
  {
    id: "msg-cdef-mnop",
    content: "Update product inventory - SKU: PRD123",
    timestamp: "2024-03-20T10:28:00Z",
    status: "pending",
    attempts: 0,
    messages: [
      {
        key: "Inventory Update Start",
        value: "Started updating product inventory"
      }
    ]
  },
  {
    id: "msg-ghij-qrst",
    content: "Process refund for order #ORD-9876",
    timestamp: "2024-03-20T10:27:30Z",
    status: "failed",
    error: "Payment gateway timeout",
    attempts: 2,
    messages: [
      {
        key: "Refund Start",
        value: "Started processing refund"
      },
      {
        key: "Payment Gateway Timeout",
        value: "Payment gateway timeout"
      }
    ]
  },
  {
    id: "msg-klmn-uvwx",
    content: "Generate invoice PDF for customer ID: CUST-456",
    timestamp: "2024-03-20T10:27:00Z",
    status: "completed",
    attempts: 1,
    messages: [
      {
        key: "Invoice PDF Start",
        value: "Started generating invoice PDF"
      },
      {
        key: "Invoice PDF Complete",
        value: "Invoice PDF generated successfully"
      }
    ]
  },
];

// Add this component for the expanded details
function EventDetails({ event }: { event: EventObject }) {
  return (
    <div className="p-4 bg-gray-50 rounded-md">
      <table className="w-full">
        <thead>
          <tr className="border-b">
            <th className="py-2 text-left text-gray-600">Key</th>
            <th className="py-2 text-left text-gray-600">Value</th>
          </tr>
        </thead>
        <tbody>
          {event.messages?.map((message, index) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: <explanation>
<tr key={index} className="border-b">
              <td className="py-2 font-medium text-gray-600">{message.key}</td>
              <td className="py-2">{message.value}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// Define columns for the messages table
const columns: ColumnDef<EventObject>[] = [
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
  /*
const { queueId }: { queueId: string } = useParams();

  const { data = mockEvents } = useQuery({
    queryKey: ["queue-messages", queueId],
    queryFn: () => [] as EventObject[],
  });
  */
  const data = mockEvents;

  return <DataTable columns={columns} data={data} renderSubComponent={({ row }) => (
    <EventDetails event={row.original} />
  )} />;
}
