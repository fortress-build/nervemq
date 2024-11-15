"use client";

import { listQueues } from "@/actions/api";
import { useQuery } from "@tanstack/react-query";
import { useRouter } from "next/navigation";

import { columns } from "@/components/queues/table";
import { DataTable } from "@/components/data-table";
import CreateQueue from "@/components/create-queue";
import { Button } from "@/components/ui/button";
import { useState } from "react";

export type Queue = {
  id: string;
  ns: string;
  name: string;
};

const mockQueues: Queue[] = [
  {
    id: "queue-1",
    ns: "default",
    name: "notifications",
  },
  {
    id: "queue-2",
    ns: "default",
    name: "emails",
  },
  {
    id: "queue-3",
    ns: "marketing",
    name: "campaign-events",
  },
  {
    id: "queue-4",
    ns: "analytics",
    name: "tracking-events",
  },
];

export default function Queues() {
  const { data = mockQueues, isLoading } = useQuery({
    //queryFn: () => listQueues(),
    queryKey: ["queues"],
  });
  const [isOpen, setIsOpen] = useState(false);
  const router = useRouter();

  return (
    <div className="flex flex-col px-4 gap-4">
      <DataTable
        className="w-full"
        columns={columns}
        data={data}
        isLoading={isLoading}
        onRowClick={(row) => router.push(`/queues/${row.id}`)}
      />

      <div className="flex justify-end">
        <Button onClick={() => setIsOpen(true)}>Create Queue</Button>
      </div>
      <CreateQueue open={isOpen} close={() => setIsOpen(false)} />
    </div>
  );
}
