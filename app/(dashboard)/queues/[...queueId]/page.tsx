"use client";
import MessageList from "@/app/(dashboard)/queues/list";
import { useQuery } from "@tanstack/react-query";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import type { QueueStatistics } from "@/components/queues/table";
import { fetchQueue } from "@/lib/actions/api";
import { QueueSettings } from "@/components/queue-settings";
import { Spinner } from "@nextui-org/spinner";
import AccessDenied from "@/components/access-denied";
import NotFound from "@/components/not-found";
import { useParams } from "next/navigation";

function Metric({
  title,
  value,
  isLoading = false,
}: {
  title: string;
  value: React.ReactNode;
  isLoading: boolean;
}) {
  return (
    <div>
      <p className="text-gray-600 break-words">{title}</p>
      {isLoading ? (
        <div className="relative flex items-center justify-start">
          <Spinner size="sm" className="absolute" />
          <p className="text-2xl font-medium opacity-0">{"0"}</p>
        </div>
      ) : (
        <p className="text-2xl font-medium">{value}</p>
      )}
    </div>
  );
}

export default function QueuePage() {
  const { queueId }: { queueId: [string, string] } = useParams();
  const [namespace, name] = queueId;

  const {
    data: queue,
    error,
    isLoading,
  } = useQuery<QueueStatistics, Error>({
    queryKey: ["queues", name, namespace],
    queryFn: () => {
      if (!name || !namespace) {
        throw new Error("Invalid queue ID");
      }
      return fetchQueue(name, namespace) as Promise<QueueStatistics>;
    },
    refetchInterval: 30000,
  });

  if (
    error !== null &&
    // FIXME: Improve error handling here
    error.message === "Access Denied"
  ) {
    return <AccessDenied returnTo={{ name: "Queues", href: "/queues" }} />;
  }

  if (queue === undefined && !isLoading) {
    return (
      <NotFound
        resource="queue"
        returnTo={{ name: "Queues", href: "/queues" }}
      />
    );
  }

  return (
    <>
      <div className="grid gap-4">
        {/* Queue Status Section */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle>Status</CardTitle>
            <QueueSettings queue={queue} />
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              <Metric
                title="Pending"
                value={queue?.pending ?? "0"}
                isLoading={isLoading}
              />
              <Metric
                title="Delivered"
                value={queue?.delivered ?? "0"}
                isLoading={isLoading}
              />
              <Metric
                title="Failed"
                value={queue?.failed ?? "0"}
                isLoading={isLoading}
              />
            </div>
          </CardContent>
        </Card>

        {/* Metrics Section */}
        <Card>
          <CardHeader>
            <CardTitle>Metrics</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              <Metric
                title="Message Size (avg)"
                value={`${(queue?.avg_size_bytes ?? 0).toFixed(2)} bytes`}
                isLoading={isLoading}
              />
              <Metric
                title="Error Rate"
                value={`${((queue?.failed ?? 0) + (queue?.delivered ?? 0) === 0 ? 0 : ((queue?.failed ?? 0) / ((queue?.delivered ?? 0) + (queue?.failed ?? 0))) * 100).toFixed(2)}%`}
                isLoading={isLoading}
              />
            </div>
          </CardContent>
        </Card>

        {/* Current Queue Items */}
        <Card>
          <CardHeader>
            <CardTitle>Messages</CardTitle>
          </CardHeader>
          <CardContent>
            <MessageList queue={name} namespace={namespace} />
          </CardContent>
        </Card>
      </div>
    </>
  );
}
