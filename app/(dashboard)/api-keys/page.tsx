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
import {
  listAPIKeys,
  deleteAPIKey,
  type DeleteTokenRequest,
} from "@/actions/api";
import { KeyToDeleteContext } from "@/lib/contexts/key-to-delete";

export default function ApiKeys() {
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [keyToDelete, setKeyToDelete] = useState<string | undefined>(undefined);

  const {
    data = [],
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ["apiKeys"],
    queryFn: () => {
      return listAPIKeys();
    },
  });

  const handleDeleteKey = async (data: DeleteTokenRequest) => {
    try {
      await deleteAPIKey(data);
      setKeyToDelete(undefined);
      await refetch();
      toast.success(`Deleted API key ${data.name}`);
    } catch {
      toast.error("Failed to delete API key");
    }
  };

  return (
    <div className="h-full flex flex-col gap-4">
      <KeyToDeleteContext.Provider
        value={{
          key: keyToDelete,
          setKey: setKeyToDelete,
        }}
      >
        <DataTable
          className="w-full"
          columns={columns}
          data={data}
          isLoading={isLoading}
        />

        <Dialog
          open={keyToDelete !== undefined}
          onOpenChange={(open) => {
            if (!open) {
              setKeyToDelete(undefined);
            }
          }}
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
                onClick={() => {
                  if (keyToDelete) {
                    handleDeleteKey({ name: keyToDelete });
                  }
                }}
              >
                Delete
              </Button>
              <Button
                variant="secondary"
                onClick={() => setKeyToDelete(undefined)}
              >
                Cancel
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </KeyToDeleteContext.Provider>

      <div className="flex justify-end">
        <Button onClick={() => setIsCreateOpen(true)}>Create API Key</Button>
      </div>

      <CreateApiKey
        open={isCreateOpen}
        close={() => setIsCreateOpen(false)}
        onSuccess={() => refetch()}
      />
    </div>
  );
}
