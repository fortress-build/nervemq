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
import type { UserStatistics } from "@/components/create-user";
import CreateUser from "@/components/create-user";
import ModifyUser from "@/components/modify-user";
import { columns } from "@/components/admin/table";
import { toast } from "sonner";
import { listUsers, deleteUser } from "@/actions/api";
import { useIsAdmin } from "@/lib/state/global";
import { redirect } from "next/navigation";
import { Input } from "@/components/ui/input";

export default function AdminPanel() {
  const isAdmin = useIsAdmin();

  if (!isAdmin) {
    redirect("/");
  }

  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [userToDelete, setUserToDelete] = useState<string | null>(null);
  const [userToModify, setUserToModify] = useState<UserStatistics | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  const {
    data = [],
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ["users", searchQuery],
    queryFn: async () => {
      const users = await listUsers();
      return users.map((user) => ({
        email: user.email,
        role: user.role,
      }));
    },
  });

  const filteredData = data.filter((user) =>
    user.email.toLowerCase().includes(searchQuery.toLowerCase())
  );

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

  const handleModifyUser = async (user: UserStatistics, e: React.MouseEvent) => {
    e.stopPropagation();
    const fullUser = {
      email: user.email,
      role: user.role,
      password: "",
      createdAt: user.createdAt,
      lastLogin: user.lastLogin,
      namespaces: user.namespaces || [],
    };
    setUserToModify(fullUser);
  };

  return (
    <div className="h-full flex flex-col gap-4">
      <div className="flex w-full max-w-sm items-center space-x-2">
        <Input
          type="text"
          placeholder="Search users..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>

      <DataTable
        className="w-full"
        columns={columns}
        data={filteredData}
        isLoading={isLoading}
        meta={{ handleDeleteUser, handleModifyUser }}
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

      <ModifyUser
        open={!!userToModify}
        close={() => setUserToModify(null)}
        onSuccess={() => {
          refetch();
          setUserToModify(null);
        }}
        user={userToModify as UserStatistics}
      />
    </div>
  );
}
