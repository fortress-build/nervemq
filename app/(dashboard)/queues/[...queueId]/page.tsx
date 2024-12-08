"use client";
import MessageList from "@/app/(dashboard)/queues/list";
import { useQuery } from "@tanstack/react-query";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
} from "@/components/ui/card";
import type { QueueStatistics } from "@/components/queues/table";
import { fetchQueue } from "@/lib/actions/api";
import { useParams, useRouter } from "next/navigation";
import { QueueSettings } from "@/components/queue-settings";
import { Button } from "@/components/ui/button";
import { Spinner } from "@nextui-org/spinner";
import { createContext, useContext } from "react";

const IsLoadingContext = createContext(false);

function Metric({ title, value }: { title: string; value: React.ReactNode }) {
  const isLoading = useContext(IsLoadingContext);

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
  const router = useRouter();
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
    return (
      <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
        <Card className="w-[400px] border">
          <CardHeader className="text-center">
            <CardTitle>Access Denied</CardTitle>
          </CardHeader>
          <CardContent className="text-center">
            <p className="mb-4">
              You don&apos;t have permission to view this queue.
            </p>
            <Button onClick={() => router.push("/queues")}>
              Return to Queues
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (queue === undefined && !isLoading) {
    return (
      <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center">
        <Card className="w-[400px] border">
          <CardHeader className="text-center">
            <CardTitle>Not Found</CardTitle>
          </CardHeader>
          <CardContent className="text-center">
            <p className="mb-4">
              The queue you are looking for does not exist.
            </p>
            <Button onClick={() => router.replace("/queues")}>
              Return to Queues
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <>
      <IsLoadingContext.Provider value={isLoading}>
        <div className="grid gap-4">
          {/* Queue Status Section */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle>Status</CardTitle>
              <QueueSettings queue={queue} />
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                <Metric title="Pending" value={queue?.pending ?? "0"} />
                <Metric title="Delivered" value={queue?.delivered ?? "0"} />
                <Metric title="Failed" value={queue?.failed ?? "0"} />
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
                />
                <Metric
                  title="Error Rate"
                  value={`${((queue?.failed ?? 0) + (queue?.delivered ?? 0) === 0 ? 0 : ((queue?.failed ?? 0) / ((queue?.delivered ?? 0) + (queue?.failed ?? 0))) * 100).toFixed(2)}%`}
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
      </IsLoadingContext.Provider>
    </>
  );
}
