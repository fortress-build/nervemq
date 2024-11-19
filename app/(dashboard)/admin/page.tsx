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
import CreateUser from "@/components/create-user";
import { columns, type User } from "@/components/admin/table";
import { toast } from "sonner";
import { listUsers, deleteUser } from "@/actions/api";

export default function AdminPanel() {
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [userToDelete, setUserToDelete] = useState<string | null>(null);

  const {
    data = [],
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ["users"],
    queryFn: async () => {
      const users = await listUsers();
      return users.map((user) => ({
        email: user.email,
        role: user.role,
        // createdAt: user.createdAt,
        // lastLogin: user.lastLogin,
        // namespaces: user.namespaces,
      }));
    },
  });

  const confirmDeleteUser = async (email: string) => {
    try {
      await deleteUser({ email });
      await refetch();
      setUserToDelete(null);
      toast.success("User deleted successfully");
    } catch {
      toast.error("Failed to delete user");
    }
  };

  const handleDeleteUser = async (email: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setUserToDelete(email);
  };

  return (
    <div className="h-full flex flex-col gap-4">
      <DataTable
        className="w-full"
        columns={columns}
        data={data.map(
          (user) =>
            ({
              ...user,
              lastLogin: (user as User).lastLogin ?? null,
            }) as User,
        )}
        isLoading={isLoading}
        meta={{ handleDeleteUser }}
      />

      <div className="flex justify-end">
        <Button onClick={() => setIsCreateOpen(true)}>Add New User</Button>
      </div>

      <CreateUser
        open={isCreateOpen}
        close={() => setIsCreateOpen(false)}
        onSuccess={() => refetch()}
      />

      <Dialog
        open={!!userToDelete}
        onOpenChange={(open) => (!open ? setUserToDelete(null) : null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete User</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this user? This action cannot be
              undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="destructive"
              onClick={async () => {
                if (userToDelete) {
                  await confirmDeleteUser(userToDelete);
                }
              }}
            >
              Delete
            </Button>
            <Button variant="secondary" onClick={() => setUserToDelete(null)}>
              Cancel
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
