"use client";

import {
  type ColumnDef,
  type SortingState,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  getFilteredRowModel,
  type ColumnFiltersState,
  useReactTable,
  getExpandedRowModel,
  type Row,
} from "@tanstack/react-table";

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";
import { Spinner } from "@nextui-org/spinner";
import React from "react";

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  className?: string;
  isLoading?: boolean;
  onRowClick?: (row: TData) => void;
  meta?: Record<string, unknown>;
  sorting?: SortingState;
  setSorting?: (sorting: SortingState) => void;
  onFilter?: (filters: ColumnFiltersState) => void;
  columnFilters?: ColumnFiltersState;
  setColumnFilters?: (filters: ColumnFiltersState) => void;
  renderSubComponent?: ({ row }: { row: Row<TData> }) => React.ReactNode;
}

export function DataTable<TData, TValue>({
  columns,
  data,
  isLoading,
  className,
  onRowClick,
  meta,
  sorting,
  setSorting,
  columnFilters,
  setColumnFilters,
  renderSubComponent,
}: DataTableProps<TData, TValue>) {
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getRowCanExpand: () => true,
    getExpandedRowModel: getExpandedRowModel(),
    ...(sorting !== undefined && {
      getSortedRowModel: getSortedRowModel(),
      onSortingChange: setSorting
        ? (updater) =>
            setSorting(
              typeof updater === "function" ? updater(sorting ?? []) : updater,
            )
        : undefined,
      state: {
        sorting: sorting ?? [],
      },
    }),
    ...(columnFilters !== undefined && {
      getFilteredRowModel: getFilteredRowModel(),
      onColumnFiltersChange: setColumnFilters
        ? (updater) =>
            setColumnFilters(
              typeof updater === "function"
                ? updater(columnFilters ?? [])
                : updater,
            )
        : undefined,
      state: {
        columnFilters: columnFilters ?? [],
      },
    }),
    state: {
      sorting: sorting ?? [],
      columnFilters: columnFilters ?? [],
    },
    meta,
  });

  return (
    <div className={cn("rounded-md border", className)}>
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => {
                return (
                  <TableHead key={header.id}>
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext(),
                        )}
                  </TableHead>
                );
              })}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows?.length ? (
            table.getRowModel().rows.map((row) => (
              <React.Fragment key={row.id}>
                <TableRow
                  data-state={row.getIsSelected() ? "selected" : undefined}
                  onClick={() => onRowClick?.(row.original)}
                  className={onRowClick ? "cursor-pointer hover:bg-muted" : ""}
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id}>
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext(),
                      )}
                    </TableCell>
                  ))}
                </TableRow>
                {row.getIsExpanded() && renderSubComponent ? (
                  <tr>
                    <td colSpan={row.getVisibleCells().length}>
                      {renderSubComponent({ row })}
                    </td>
                  </tr>
                ) : null}
              </React.Fragment>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                {isLoading ? <Spinner /> : "No results."}
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}
