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
import { columns } from "@/components/api-keys/table";
import { toast } from "sonner";
import { listAPIKeys, deleteAPIKey } from "@/actions/api";

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
        // id: key.id,
        name: key.name,
        // createdAt: key.created_at,
        // lastUsed: key.last_used,
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
