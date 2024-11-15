"use client";
import { listNamespaces } from "@/actions/api";
import { columns } from "@/components/namespaces/table";
import CreateNamespace from "@/components/create-namespace";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { DataTable } from "@/components/data-table";
import { ChevronRight, Trash2 } from "lucide-react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog";
import { deleteNamespace } from "@/actions/api";

export default function Namespaces() {
  const [isOpen, setIsOpen] = useState(false);
  const { data = [], isLoading, refetch } = useQuery({
    queryKey: ["namespaces"],
    queryFn: () => listNamespaces(),
  });
  const [namespaceToDelete, setNamespaceToDelete] = useState<string | null>(null);

  const handleDeleteNamespace = async (name: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setNamespaceToDelete(name);
  };

  return (
    <div className="h-full flex flex-col gap-4">
      <DataTable
        className="w-full"
        columns={[
          {
            id: "chevron",
            cell: () => (
              <ChevronRight className="h-4 w-4 text-muted-foreground" />
            ),
          },
          ...columns,
          {
            id: "actions",
            cell: (row) => (
              <div className="flex items-center justify-end gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  className="text-destructive hover:text-destructive hover:bg-destructive/10"
                  onClick={(e) => handleDeleteNamespace(row.row.original.name, e)}
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            ),
          },
        ]}
        data={data}
        isLoading={isLoading}
      />
      <div className="flex justify-end">
        <Button onClick={() => setIsOpen(true)}>Create Namespace</Button>
      </div>
      <CreateNamespace open={isOpen} close={() => setIsOpen(false)} />
      <Dialog open={!!namespaceToDelete} onOpenChange={(open) => !open && setNamespaceToDelete(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Namespace</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this namespace? This action cannot be undone.
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
