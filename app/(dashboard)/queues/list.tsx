"use client";

import { DataTable } from "@/components/data-table";
import type { ColumnDef } from "@tanstack/react-table";
import { ChevronDown, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";

type KVPair = {
  key: string;
  value: string;
};

type MessageObject = {
  id: string;
  content: string;
  timestamp: string;
  status: "pending" | "completed" | "failed";
  error?: string;
  attempts?: number;
  messages?: KVPair[];
};

// Mock data for development
const mockMessages: MessageObject[] = [
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

// Updated component name and prop type
function MessageDetails({ message }: { message: MessageObject }) {
  return (
    <div className="p-6 space-y-4 bg-gray-50">
      <h3 className="font-semibold text-gray-700 mb-2">Message Details</h3>
      <div className="grid gap-3">
        {message.messages?.map((message, index) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: <explanation>
          <div key={index} className="bg-white p-4 rounded-lg border border-gray-200">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <span className="text-xs uppercase text-gray-400">Key</span>
                <div className="mt-1 text-sm font-medium text-gray-700">{message.key}</div>
              </div>
              <div>
                <span className="text-xs uppercase text-gray-400">Value</span>
                <div className="mt-1 text-sm text-gray-700">{message.value}</div>
              </div>
            </div>
          </div>
        ))}
      </div>
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
    accessorKey: "content",
    header: "Message",
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
];

export default function ClientList() {
  const data = mockMessages;
    /*
const { queueId }: { queueId: string } = useParams();

  const { data = mockMessages } = useQuery({
    queryKey: ["queue-messages", queueId],
    queryFn: () => [] as MessageObject[],
  });
  */

  return <DataTable columns={columns} data={data} renderSubComponent={({ row }) => (
    <MessageDetails message={row.original} />
  )} />;
}
