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
import type { SortingState } from "@tanstack/react-table";

export default function AdminPanel() {
  // Authentication check
  const isAdmin = useIsAdmin();
  if (!isAdmin) {
    redirect("/");
  }

  const [isCreateOpen, setIsCreateOpen] = useState(false);        // Controls Create User modal
  const [userToDelete, setUserToDelete] = useState<string | undefined>(undefined);  // Controls Delete User dialog
  const [userToModify, setUserToModify] = useState<UserStatistics | undefined>(undefined);  // Controls Modify User modal
  const [searchQuery, setSearchQuery] = useState("");             // Search input state
  const [sorting, setSorting] = useState<SortingState>([]);       // Table sorting state

  // Fetch and filter users data
  const {
    data = [],
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ["users", searchQuery],
    queryFn: () => listUsers(),
    select: (data) =>
      data.filter((user) =>
        user.email.toLowerCase().includes(searchQuery.toLowerCase()),
      ),
  });

  // User management handlers
  const confirmDeleteUser = async (email: string) => {
    try {
      await deleteUser({ email });
      await refetch();
      setUserToDelete(undefined);
      toast.success("User deleted successfully");
    } catch {
      toast.error("Failed to delete user");
    }
  };

  const handleDeleteUser = async (email: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setUserToDelete(email);
  };

  const handleModifyUser = async (
    user: UserStatistics,
    e: React.MouseEvent,
  ) => {
    e.stopPropagation();
    const fullUser = {
      email: user.email,
      role: user.role,
      createdAt: user.createdAt,
      lastLogin: user.lastLogin,
    };
    setUserToModify(fullUser);
  };

  return (
    <div className="h-full flex flex-col gap-4">
      {/* Search Bar */}
      <div className="flex w-full max-w-sm items-center space-x-2">
        <Input
          type="text"
          placeholder="Search users..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>

      {/* Users Data Table */}
      <DataTable
        className="w-full"
        columns={columns}
        data={data}
        isLoading={isLoading}
        meta={{ handleDeleteUser, handleModifyUser }}
        sorting={sorting}
        setSorting={setSorting}
      />

      {/* Add New User Button */}
      <div className="flex justify-end">
        <Button onClick={() => setIsCreateOpen(true)}>Add New User</Button>
      </div>

      {/* Modals and Dialogs */}
      <CreateUser
        open={isCreateOpen}
        close={() => setIsCreateOpen(false)}
        onSuccess={() => refetch()}
      />

      <Dialog
        open={!!userToDelete}
        onOpenChange={(open) => (!open ? setUserToDelete(undefined) : null)}
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
            <Button
              variant="secondary"
              onClick={() => setUserToDelete(undefined)}
            >
              Cancel
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ModifyUser
        open={!!userToModify}
        close={() => setUserToModify(undefined)}
        onSuccess={() => {
          refetch();
          setUserToModify(undefined);
        }}
        user={userToModify as UserStatistics}
      />
    </div>
  );
}
