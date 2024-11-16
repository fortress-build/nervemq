"use client";

import { listQueues } from "@/actions/api";
import { useQuery } from "@tanstack/react-query";
import { useRouter } from "next/navigation";

import { columns, type QueueStatistics } from "@/components/queues/table";
import { DataTable } from "@/components/data-table";
import CreateQueue from "@/components/create-queue";
import { Button } from "@/components/ui/button";
import { useState } from "react";
import { deleteQueue } from "@/actions/api";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";

export type Queue = {
  id: string;
  ns: string;
  name: string;
};

export default function Queues() {
  const {
    data = [],
    isLoading,
    refetch,
  } = useQuery({
    queryFn: () => listQueues(),
    queryKey: ["queues"],
  });
  const [isOpen, setIsOpen] = useState(false);
  const router = useRouter();
  const [queueToDelete, setQueueToDelete] = useState<{
    name: string;
    ns: string;
  } | null>(null);

  const handleDeleteQueue = async (
    name: string,
    ns: string,
    e: React.MouseEvent,
  ) => {
    e.stopPropagation();
    setQueueToDelete({ name, ns });
  };

  return (
    <div className="flex flex-col gap-4">
      <DataTable
        className="w-full"
        columns={columns}
        data={data}
        isLoading={isLoading}
        onRowClick={(row: QueueStatistics) =>
          router.push(`/queues/${row.name}`)
        }
        meta={{ handleDeleteQueue }}
      />

      <div className="flex justify-end">
        <Button onClick={() => setIsOpen(true)}>Create Queue</Button>
      </div>
      <CreateQueue open={isOpen} close={() => setIsOpen(false)} />

      <Dialog
        open={!!queueToDelete}
        onOpenChange={(open) => !open && setQueueToDelete(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Queue</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this queue? This action cannot be
              undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="destructive"
              onClick={async () => {
                if (queueToDelete) {
                  await deleteQueue({
                    name: queueToDelete.name,
                    namespace: queueToDelete.ns,
                  });
                  refetch();
                  setQueueToDelete(null);
                }
              }}
            >
              Delete
            </Button>
            <Button variant="secondary" onClick={() => setQueueToDelete(null)}>
              Cancel
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}