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
import { queueSettingsSchema, type QueueSettingsType } from "@/schemas/queue-settings";

export function QueueSettings({ namespace, queue }: { namespace: string; queue: string }) {
  const [open, setOpen] = useState(false);
  const queryClient = useQueryClient();

  const { data: settings, isLoading } = useQuery({
    queryKey: ["queueSettings", namespace, queue],
    queryFn: () => getQueueSettings(namespace, queue),
    enabled: open,
  });

  const { mutate: saveSettings, isPending } = useMutation({
    mutationFn: updateQueueSettings,
    onSuccess: () => {
      toast.success("Settings updated successfully");
      queryClient.invalidateQueries({ queryKey: ["queueSettings", namespace, queue] });
      setOpen(false);
    },
    onError: (error: Error) => {
      toast.error(error.message || "Failed to update settings");
    },
  });

  const form = useForm<QueueSettingsType>({
    defaultValues: {
      namespace,
      queue,
      maxRetries: 3,
      timeout: 30,
      batchSize: 100,
    },
    onSubmit: async (values) => {
      try {
        await queueSettingsSchema.validate(values, { abortEarly: false });
        saveSettings(values);
      } catch (error) {
        if (error instanceof Error) {
          form.setErrors(toFormErrors(error));
        }
      }
    },
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
          <form.Provider>
            <form onSubmit={(e) => {
              e.preventDefault();
              form.handleSubmit();
            }}>
              <div className="grid gap-4 py-4">
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="retries" className="text-right">
                    Max Retries
                  </Label>
                  <form.Field name="maxRetries">
                    {(field) => (
                      <Input
                        id="retries"
                        type="number"
                        className="col-span-3"
                        value={field.value}
                        onChange={(e) => field.setValue(parseInt(e.target.value))}
                        error={field.error}
                      />
                    )}
                  </form.Field>
                </div>
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="timeout" className="text-right">
                    Timeout (s)
                  </Label>
                  <form.Field name="timeout">
                    {(field) => (
                      <Input
                        id="timeout"
                        type="number"
                        className="col-span-3"
                        value={field.value}
                        onChange={(e) => field.setValue(parseInt(e.target.value))}
                        error={field.error}
                      />
                    )}
                  </form.Field>
                </div>
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="batch-size" className="text-right">
                    Batch Size
                  </Label>
                  <form.Field name="batchSize">
                    {(field) => (
                      <Input
                        id="batch-size"
                        type="number"
                        className="col-span-3"
                        value={field.value}
                        onChange={(e) => field.setValue(parseInt(e.target.value))}
                        error={field.error}
                      />
                    )}
                  </form.Field>
                </div>
              </div>
              <DialogFooter>
                <Button type="submit" disabled={isPending}>
                  {isPending ? "Saving..." : "Save Changes"}
                </Button>
              </DialogFooter>
            </form>
          </form.Provider>
        )}
      </DialogContent>
    </Dialog>
  );
}
