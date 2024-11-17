"use client";
import type { ColumnDef } from "@tanstack/react-table";
import { Trash2, User, Mail, Shield } from "lucide-react";
import { Button } from "../ui/button";

// biome-ignore lint/suspicious/noRedeclare: <explanation>
export type User = {
  id: string;
  name: string;
  email: string;
  role: string;
  createdAt: string;
  lastLogin: string | null;
  namespaces: string[];
};

export type UserStatistics = User;

export const columns: ColumnDef<UserStatistics>[] = [
  // {
  //   accessorKey: "name",
  //   header: () => (
  //     <div className="flex items-center gap-2">
  //       <User className="h-4 w-4" />
  //       <span>Name</span>
  //     </div>
  //   ),
  // },
  {
    accessorKey: "email",
    header: () => (
      <div className="flex items-center gap-2">
        <Mail className="h-4 w-4" />
        <span>Email</span>
      </div>
    ),
  },
  {
    accessorKey: "role",
    header: () => (
      <div className="flex items-center gap-2">
        <Shield className="h-4 w-4" />
        <span>Role</span>
      </div>
    ),
  },
  // {
  //   accessorKey: "createdAt",
  //   header: () => (
  //     <div className="flex items-center gap-2">
  //       <Calendar className="h-4 w-4" />
  //       <span>Joined</span>
  //     </div>
  //   ),
  //   cell: ({ row }) => new Date(row.original.createdAt).toLocaleDateString(),
  // },
  // {
  //   accessorKey: "lastLogin",
  //   header: () => (
  //     <div className="flex items-center gap-2">
  //       <Clock className="h-4 w-4" />
  //       <span>Last Login</span>
  //     </div>
  //   ),
  //   cell: ({ row }) =>
  //     row.original.lastLogin
  //       ? new Date(row.original.lastLogin).toLocaleDateString()
  //       : "Never",
  // },
  {
    id: "actions",
    cell: (row) => (
      <div className="flex items-center justify-end gap-2">
        <Button
          variant="ghost"
          size="sm"
          className="text-destructive hover:text-destructive hover:bg-destructive/10"
          onClick={async (e) => {
            const meta = row.table.options.meta as
              | {
                  handleDeleteUser: (id: string, e: unknown) => void;
                }
              | undefined;
            meta?.handleDeleteUser(row.row.original.id, e);
          }}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    ),
  },
];
