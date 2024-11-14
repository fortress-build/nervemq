"use server";

import { listQueues } from "@/actions/api";
import { QueuesTable } from "@/components/queues/table";


export default async function Queues() {
  const data = await listQueues();
  console.log(data);

  return (
    <div className="h-full flex">
      <QueuesTable data={data} />
    </div>
  );
}
