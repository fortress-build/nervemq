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
import CreateUser, { type UserStatistics } from "@/components/create-user";
import type { ColumnDef } from "@tanstack/react-table";
import { Trash2 } from "lucide-react";
import { toast } from "sonner";
import { listUsers, deleteUser } from "@/actions/api";

const columns: ColumnDef<UserStatistics>[] = [
  {
    accessorKey: "email",
    header: "Email",
  },
  {
    accessorKey: "role",
    header: "Role",
  },
  // {
  //   accessorKey: "createdAt",
  //   header: "Joined",
  //   cell: ({ row }) => new Date(row.original.createdAt).toLocaleDateString(),
  // },
  // {
  //   accessorKey: "lastLogin",
  //   header: "Last Login",
  //   cell: ({ row }) =>
  //     row.original.lastLogin
  //       ? new Date(row.original.lastLogin).toLocaleDateString()
  //       : "Never",
  // },
  {
    id: "actions",
    cell: () => {
      // const meta = table.options.meta as {
      //   handleDeleteUser: (id: string) => void;
      // };

      // { row, table }

      return (
        <Button
          variant="ghost"
          size="icon"
          onClick={(e) => {
            e.stopPropagation();
            // meta.handleDeleteUser(row.original.id);
          }}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      );
    },
  },
];

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
        // id: user.id,
        email: user.email,
        // role: user.role,
        // createdAt: user.createdAt,
        // lastLogin: user.lastLogin,
        // namespaces: user.namespaces,
      }));
    },
  });

  const handleDeleteUser = async (id: string) => {
    try {
      await deleteUser(id);
      await refetch();
      setUserToDelete(null);
      toast.success("User deleted successfully");
    } catch {
      toast.error("Failed to delete user");
    }
  };

  return (
    <div className="h-full flex flex-col gap-4">
      <DataTable
        className="w-full"
        columns={columns}
        data={data}
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
        onOpenChange={(open) => !open && setUserToDelete(null)}
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
              onClick={() => userToDelete && handleDeleteUser(userToDelete)}
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
