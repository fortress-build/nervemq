"use client";

import { listQueues } from "@/actions/api";
import { useQuery } from "@tanstack/react-query";

import { columns } from "@/components/queues/table";
import { DataTable } from "@/components/data-table";

export default function Queues() {
  const { data = [], isLoading } = useQuery({
    queryFn: () => listQueues(),
    queryKey: ["queues"],
  });

  return (
    <div className="flex px-4">
      <DataTable
        className="w-full"
        columns={columns}
        data={data}
        isLoading={isLoading}
      />
    </div>
  );
}
