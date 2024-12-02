"use client";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ChevronsUpDown, Settings2 } from "lucide-react";
import {
  getQueueSettings,
  listQueues,
  updateQueueSettings,
} from "@/actions/api";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { useState } from "react";
import { useForm } from "@tanstack/react-form";
import { type YupValidator, yupValidator } from "@tanstack/yup-form-adapter";
import {
  type QueueConfig,
  type UpdateQueueConfigRequest,
  updateQueueConfigSchema,
} from "@/schemas/queue-settings";
import type { QueueStatistics } from "./queues/table";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";
import { cn } from "@/lib/utils";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "./ui/command";
import { Spinner } from "@nextui-org/react";

export function QueueSettings({ queue }: { queue?: QueueStatistics }) {
  const [open, setOpen] = useState(false);
  const [dlqPopoverOpen, setDlqPopoverOpen] = useState(false);
  const queryClient = useQueryClient();

  const { data: settings, isLoading } = useQuery({
    queryKey: ["queueSettings"],
    queryFn: () => getQueueSettings(queue?.ns, queue?.name),
  });

  const { mutate: saveSettings, isPending } = useMutation<
    unknown,
    Error,
    UpdateQueueConfigRequest
  >({
    mutationFn: (data: UpdateQueueConfigRequest) => updateQueueSettings(data),
    onSuccess: () => {
      toast.success("Settings updated successfully");
      queryClient.invalidateQueries({
        queryKey: ["queueSettings"],
      });
      setOpen(false);
    },
    onError: (error: Error) => {
      toast.error(error.message || "Failed to update settings");
    },
  });

  const form = useForm<QueueConfig, YupValidator>({
    defaultValues: {
      maxRetries: settings?.maxRetries ?? 0,
      deadLetterQueue: settings?.deadLetterQueue ?? undefined,
    },
    validatorAdapter: yupValidator(),
    validators: {
      onChange: updateQueueConfigSchema,
      onMount: updateQueueConfigSchema,
    },
    onSubmit: ({ value }) => {
      if (queue === undefined) {
        return;
      }
      return saveSettings({
        ...value,
        queue: queue?.name,
        namespace: queue?.ns,
      });
    },
  });

  const { data: availableQueues = [], isLoading: queuesLoading } = useQuery({
    queryFn: () => listQueues(),
    queryKey: ["queues"],
    select: (data) =>
      Array.from(data.values()).filter(
        (queue: QueueStatistics) => queue.ns === queue?.ns,
      ),
  });

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button
          size="icon"
          variant={"ghost"}
          className="transition-all"
          disabled={queue === undefined}
        >
          <Settings2 className="h-4 w-4" />
        </Button>
      </DialogTrigger>
      <DialogContent className="rounded-lg sm:rounded-lg">
        <DialogHeader>
          <DialogTitle>Queue Settings</DialogTitle>
        </DialogHeader>
        {isLoading ? (
          <div className="flex items-center justify-center min-h-24">
            <Spinner />
          </div>
        ) : (
          <form
            onSubmit={(e) => {
              e.preventDefault();
              e.stopPropagation();
              void form.handleSubmit();
            }}
          >
            <div className="grid gap-4 py-4">
              <form.Field name="maxRetries">
                {(field) => (
                  <div className="flex flex-col gap-2">
                    <Label htmlFor={field.name}>Max Retries</Label>
                    <Input
                      id={field.name}
                      type="number"
                      className="col-span-3"
                      value={field.state.value}
                      onChange={(e) =>
                        field.handleChange(Number.parseInt(e.target.value))
                      }
                      onBlur={field.handleBlur}
                    />
                    {field.state.meta.errors ? (
                      <span className="col-start-2 col-span-3 text-sm text-destructive">
                        {field.state.meta.errors.join(", ")}
                      </span>
                    ) : null}
                  </div>
                )}
              </form.Field>

              <form.Field name="deadLetterQueue">
                {(field) => (
                  <div className="flex flex-col gap-2">
                    <Label htmlFor={field.name}>Dead Letter Queue</Label>
                    <Popover
                      open={dlqPopoverOpen}
                      onOpenChange={setDlqPopoverOpen}
                    >
                      <PopoverTrigger asChild>
                        <Button
                          variant="outline"
                          onBlur={field.handleBlur}
                          className={cn(
                            "w-full justify-between col-span-3",
                            field.state.value ? "" : "text-muted-foreground",
                          )}
                        >
                          {field.state.value
                            ? field.state.value
                            : "Select dead letter queue"}
                          <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                        </Button>
                      </PopoverTrigger>
                      <PopoverContent className="w-[var(--radix-popover-trigger-width)] p-0">
                        <Command
                          onBlur={field.handleBlur}
                          value={field.state.value}
                          onValueChange={field.handleChange}
                          className="bg-background"
                        >
                          <CommandInput placeholder="Search queues..." />
                          <CommandList>
                            <CommandEmpty>
                              {queuesLoading ? (
                                <Spinner />
                              ) : (
                                <div className="flex flex-col items-center justify-center py-4 gap-2">
                                  <p className="text-sm text-muted-foreground">
                                    No queues found.
                                  </p>
                                </div>
                              )}
                            </CommandEmpty>
                            <CommandGroup>
                              {availableQueues.map((queue) => (
                                <CommandItem
                                  key={queue.name}
                                  value={queue.name}
                                  className="cursor-pointer"
                                  onSelect={(currentValue) => {
                                    field.handleChange(currentValue);
                                    setDlqPopoverOpen(false);
                                  }}
                                >
                                  {queue.name}
                                </CommandItem>
                              ))}
                            </CommandGroup>
                          </CommandList>
                        </Command>
                      </PopoverContent>
                    </Popover>
                    {field.state.meta.errors ? (
                      <span className="text-sm text-destructive">
                        {field.state.meta.errors.join(", ")}
                      </span>
                    ) : null}
                  </div>
                )}
              </form.Field>
            </div>

            <DialogFooter className="gap-2">
              <Button
                variant="outline"
                type="button"
                onClick={() => setOpen(false)}
              >
                Cancel
              </Button>
              <form.Subscribe
                selector={(state) => [state.isValid, state.isSubmitting]}
              >
                {([isValid]) => (
                  <Button type="submit" disabled={!isValid}>
                    {isPending ? "Saving..." : "Save Changes"}
                  </Button>
                )}
              </form.Subscribe>
            </DialogFooter>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}
