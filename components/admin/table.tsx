"use client";
import type { ColumnDef } from "@tanstack/react-table";
import { Trash2, Mail, Shield, Pencil, ArrowUpDown } from "lucide-react";
import { Button } from "../ui/button";
import type { UserStatistics } from "../create-user";

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
    header: ({ column }) => (
      <div className="flex items-center gap-2">
        <Mail className="h-4 w-4" />
        <Button
          variant="ghost"
          className="p-0 hover:bg-transparent"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          <span>Email</span>
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      </div>
    ),
    enableSorting: true,
  },
  {
    accessorKey: "role",
    header: ({ column }) => (
      <div className="flex items-center gap-2">
        <Shield className="h-4 w-4" />
        <Button
          variant="ghost"
          className="p-0 hover:bg-transparent"
          onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        >
          <span>Role</span>
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      </div>
    ),
    enableSorting: true,
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
          className="hover:bg-secondary/80"
          onClick={async (e) => {
            const meta = row.table.options.meta as
              | {
                  handleModifyUser: (email: string, e: unknown) => void;
                }
              | undefined;
            meta?.handleModifyUser(row.row.original.email, e);
          }}
        >
          <Pencil className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="text-destructive hover:text-destructive hover:bg-destructive/10"
          onClick={async (e) => {
            const meta = row.table.options.meta as
              | {
                  handleDeleteUser: (email: string, e: unknown) => void;
                }
              | undefined;
            meta?.handleDeleteUser(row.row.original.email, e);
          }}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    ),
  },
];
