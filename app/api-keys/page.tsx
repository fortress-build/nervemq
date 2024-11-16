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

type ApiKey = {
  id: string;
  name: string;
  createdAt: string;
  lastUsed?: string;
};

const columns = [
  {
    accessorKey: "name",
    header: "Name",
  },
  {
    accessorKey: "createdAt",
    header: "Created",
  },
  {
    accessorKey: "lastUsed",
    header: "Last Used",
  },
];

export default function ApiKeys() {
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  
  const { data = [], isLoading } = useQuery({
    queryKey: ["apiKeys"],
    queryFn: async () => {
      // TODO: Implement API key fetching
      return [] as ApiKey[];
    },
  });

  return (
    <div className="flex flex-col gap-4">
      <DataTable
        columns={columns}
        data={data}
        isLoading={isLoading}
      />

      <div className="flex justify-end">
        <Button onClick={() => setIsCreateOpen(true)}>
          Create API Key
        </Button>
      </div>

      <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create API Key</DialogTitle>
            <DialogDescription>
              Create a new API key to access the API programmatically.
            </DialogDescription>
          </DialogHeader>
          
          {/* TODO: Add form for creating API key */}
          
          <DialogFooter>
            <Button variant="secondary" onClick={() => setIsCreateOpen(false)}>
              Cancel
            </Button>
            <Button onClick={() => {
              // TODO: Implement API key creation
              setIsCreateOpen(false);
            }}>
              Create
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
