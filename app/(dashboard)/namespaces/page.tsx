"use client";
import { listNamespaces } from "@/lib/actions/api";
import { columns } from "@/lib/components/namespaces/table";
import CreateNamespace from "@/lib/components/create-namespace";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "@/lib/components/ui/button";
import { DataTable } from "@/lib/components/data-table";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/lib/components/ui/dialog";
import { deleteNamespace } from "@/lib/actions/api";
import { Input } from "@/lib/components/ui/input";
import type { SortingState } from "@tanstack/react-table";

export default function Namespaces() {
  const [isOpen, setIsOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const {
    data = [],
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ["namespaces", searchQuery],
    queryFn: () => listNamespaces(),
  });
  const [namespaceToDelete, setNamespaceToDelete] = useState<string | null>(
    null,
  );
  const [sorting, setSorting] = useState<SortingState>([]);

  const handleDeleteNamespace = async (name: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setNamespaceToDelete(name);
  };

  const filteredData = data.filter((namespace) =>
    namespace.name.toLowerCase().includes(searchQuery.toLowerCase()),
  );

  return (
    <div className="h-full flex flex-col gap-4">
      <div className="flex w-full max-w-sm items-center space-x-2">
        <Input
          type="text"
          placeholder="Search namespaces..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>
      <DataTable
        className="w-full"
        columns={columns}
        data={filteredData}
        isLoading={isLoading}
        meta={{ handleDeleteNamespace }}
        sorting={sorting}
        setSorting={setSorting}
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
