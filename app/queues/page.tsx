"use client";

import { listQueues } from "@/actions/api";
import { useQuery } from "@tanstack/react-query";
import { useRouter } from "next/navigation";

import { columns, type QueueStatistics } from "@/components/queues/table";
import { DataTable } from "@/components/data-table";
import CreateQueue from "@/components/create-queue";
import { Button } from "@/components/ui/button";
import { useState } from "react";
import { ChevronRight } from "lucide-react";

export type Queue = {
  id: string;
  ns: string;
  name: string;
};

export default function Queues() {
  const { data = [], isLoading } = useQuery({
    queryFn: () => listQueues(),
    queryKey: ["queues"],
  });
  const [isOpen, setIsOpen] = useState(false);
  const router = useRouter();

  return (
    <div className="flex flex-col gap-4">
      <DataTable
        className="w-full"
        columns={[
          {
            id: "actions",
            cell: () => <ChevronRight className="h-4 w-4 text-muted-foreground" />,
          },
          ...columns,
        ]}
        data={data}
        isLoading={isLoading}
        onRowClick={(row: QueueStatistics) => router.push(`/queues/${row.id}`)}
      />

      <div className="flex justify-end">
        <Button onClick={() => setIsOpen(true)}>Create Queue</Button>
      </div>
      <CreateQueue open={isOpen} close={() => setIsOpen(false)} />
    </div>
  );
}
