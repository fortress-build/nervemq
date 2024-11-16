"use client";
import { listNamespaces } from "@/actions/api";
import { columns } from "@/components/namespaces/table";
import CreateNamespace from "@/components/create-namespace";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
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
import { deleteNamespace } from "@/actions/api";

export default function Namespaces() {
  const [isOpen, setIsOpen] = useState(false);
  const {
    data = [],
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ["namespaces"],
    queryFn: () => listNamespaces(),
  });
  const [namespaceToDelete, setNamespaceToDelete] = useState<string | null>(
    null,
  );

  const handleDeleteNamespace = async (name: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setNamespaceToDelete(name);
  };

  return (
    <div className="h-full flex flex-col gap-4">
      <DataTable
        className="w-full"
        columns={columns}
        data={data}
        isLoading={isLoading}
        meta={{ handleDeleteNamespace }}
      />
      <div className="flex justify-end">
        <Button onClick={() => setIsOpen(true)}>Create Namespace</Button>
      </div>
      <CreateNamespace open={isOpen} close={() => setIsOpen(false)} />
      <Dialog
        open={!!namespaceToDelete}
        onOpenChange={(open) => !open && setNamespaceToDelete(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Namespace</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this namespace? This action cannot
              be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="destructive"
              onClick={async () => {
                if (namespaceToDelete) {
                  await deleteNamespace(namespaceToDelete);
                  refetch();
                  setNamespaceToDelete(null);
                }
              }}
            >
              Delete
            </Button>
            <Button
              variant="secondary"
              onClick={() => setNamespaceToDelete(null)}
            >
              Cancel
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
