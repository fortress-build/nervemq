"use client";

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { DataTable } from "@/components/data-table";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import CreateApiKey from "@/components/create-api-key";
import type { ColumnDef } from "@tanstack/react-table";
import { Trash2 } from "lucide-react";
import { toast } from "sonner";
import { listAPIKeys, deleteAPIKey } from "@/actions/api";

type ApiKey = {
  id: string;
  name: string;
  createdAt: string;
  lastUsed?: string;
};

const columns: ColumnDef<ApiKey>[] = [
  {
    accessorKey: "name",
    header: "Name",
  },
  {
    accessorKey: "createdAt",
    header: "Created",
    cell: ({ row }) => new Date(row.original.createdAt).toLocaleDateString(),
  },
  {
    accessorKey: "lastUsed",
    header: "Last Used",
    cell: ({ row }) =>
      row.original.lastUsed
        ? new Date(row.original.lastUsed).toLocaleDateString()
        : "Never",
  },
  {
    id: "actions",
    cell: ({ row, table }) => {
      const meta = table.options.meta as {
        handleDeleteKey: (id: string) => void;
      };

      return (
        <Button
          variant="ghost"
          size="icon"
          onClick={(e) => {
            e.stopPropagation();
            meta.handleDeleteKey(row.original.id);
          }}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      );
    },
  },
];

export default function ApiKeys() {
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [keyToDelete, setKeyToDelete] = useState<string | null>(null);

  const {
    data = [],
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ["apiKeys"],
    queryFn: async () => {
      const keys = await listAPIKeys();
      return keys.map((key) => ({
        id: key.id,
        name: key.name,
        createdAt: key.created_at,
        lastUsed: key.last_used,
      }));
    },
  });

  const handleDeleteKey = async (id: string) => {
    try {
      await deleteAPIKey(id);
      await refetch();
      setKeyToDelete(null);
    } catch {
      toast.error("Failed to delete API key");
    }
  };

  return (
    <div className="h-full flex flex-col gap-4">
      <DataTable
        className="w-full"
        columns={columns}
        data={data}
        isLoading={isLoading}
        meta={{ handleDeleteKey }}
      />

      <div className="flex justify-end">
        <Button onClick={() => setIsCreateOpen(true)}>Create API Key</Button>
      </div>

      <CreateApiKey
        open={isCreateOpen}
        close={() => setIsCreateOpen(false)}
        onSuccess={() => refetch()}
      />

      <Dialog
        open={!!keyToDelete}
        onOpenChange={(open) => !open && setKeyToDelete(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete API Key</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this API key? This action cannot
              be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="destructive"
              onClick={() => keyToDelete && handleDeleteKey(keyToDelete)}
            >
              Delete
            </Button>
            <Button variant="secondary" onClick={() => setKeyToDelete(null)}>
              Cancel
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
