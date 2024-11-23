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
import { Settings2 } from "lucide-react";
import { getQueueSettings, updateQueueSettings } from "@/actions/api";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { useState, useEffect } from "react";
import { useForm } from "@tanstack/react-form";
import { yupValidator } from "@tanstack/yup-form-adapter";
import { queueSettingsSchema } from "@/schemas/queue-settings";
import type { QueueStatistics } from "./queues/table";

export function QueueSettings({ queue }: { queue?: QueueStatistics }) {
  const [open, setOpen] = useState(false);
  const queryClient = useQueryClient();

  const { data: settings, isLoading } = useQuery({
    queryKey: [
      "queueSettings",
      {
        ns: queue?.ns,
        name: queue?.name,
      },
    ],
    queryFn: () => getQueueSettings(queue?.ns, queue?.name),
    enabled: open,
  });

  const { mutate: saveSettings, isPending } = useMutation({
    mutationFn: updateQueueSettings,
    onSuccess: () => {
      toast.success("Settings updated successfully");
      queryClient.invalidateQueries({
        queryKey: [
          "queueSettings",
          {
            ns: queue?.ns,
            queue: queue?.name,
          },
        ],
      });
      setOpen(false);
    },
    onError: (error: Error) => {
      toast.error(error.message || "Failed to update settings");
    },
  });

  const form = useForm({
    defaultValues: {
      namespace: queue?.ns ?? "",
      queue: queue?.name ?? "",
      maxRetries: 3,
      timeout: 30,
    },
    validatorAdapter: yupValidator(),
    validators: {
      onChange: queueSettingsSchema,
      onMount: queueSettingsSchema,
    },
    onSubmit: ({ value }) => saveSettings(value),
  });

  // Update form values when settings are loaded
  useEffect(() => {
    if (settings) {
      form.reset(settings);
    }
  }, [settings, form]);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="icon">
          <Settings2 className="h-4 w-4" />
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Queue Settings</DialogTitle>
        </DialogHeader>
        {isLoading ? (
          <div>Loading...</div>
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
                  <div className="grid grid-cols-4 items-center gap-4">
                    <Label htmlFor={field.name} className="text-right">
                      Max Retries
                    </Label>
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

              <form.Field name="timeout">
                {(field) => (
                  <div className="grid grid-cols-4 items-center gap-4">
                    <Label htmlFor={field.name} className="text-right">
                      Timeout (s)
                    </Label>
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
                selector={(state) => [state.canSubmit, state.isSubmitting]}
              >
                {([canSubmit]) => (
                  <Button type="submit" disabled={!canSubmit || isPending}>
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
