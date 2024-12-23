import { Button } from "./ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";

import { useForm } from "@tanstack/react-form";
import { zodValidator, type ZodValidator } from "@tanstack/zod-form-adapter";
import { useQuery } from "@tanstack/react-query";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { cn } from "@/lib/utils";
import { createQueue, listNamespaces } from "@/lib/actions/api";
import { Spinner } from "@nextui-org/react";
import { ChevronsUpDown, Plus } from "lucide-react";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "./ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";
import { toast } from "sonner";
import { useInvalidate } from "@/lib/hooks/use-invalidate";
import CreateNamespace from "./create-namespace";
import { useState, useEffect } from "react";
import {
  type CreateQueueRequest,
  createQueueSchema,
} from "@/lib/schemas/create-queue";
import KeyValueForm from "./key-value-pairs";

export default function CreateQueue({
  open,
  close,
}: {
  open: boolean;
  close: () => void;
}) {
  const [showCreateNamespace, setShowCreateNamespace] = useState(false);
  const [nsPopoverOpen, setNsPopoverOpen] = useState(false);

  const { data: namespaces = [], isLoading } = useQuery({
    queryFn: () => listNamespaces(),
    queryKey: ["namespaces"],
  });

  const invalidate = useInvalidate(["queues"]);

  const form = useForm<CreateQueueRequest, ZodValidator>({
    defaultValues: {
      name: "",
      namespace: "",
      attributes: new Map(),
      tags: new Map(),
    },
    validatorAdapter: zodValidator(),
    validators: {
      onChange: createQueueSchema,
      onMount: createQueueSchema,
    },
    onSubmit: async ({ value: data, formApi }) => {
      await createQueue(data)
        .then(() => {
          invalidate();
        })
        .catch(() => {
          toast.error("Something went wrong");
        })
        .finally(() => {
          close();
          formApi.reset();
        });
    },
  });

  const handleNamespaceCreated = async (namespaceName: string) => {
    form.setFieldValue("namespace", namespaceName);
    await form.validateField("namespace", "change");
    setShowCreateNamespace(false);
  };

  useEffect(() => {
    if (namespaces.length === 1) {
      form.setFieldValue("namespace", namespaces[0].name);
    }
  }, [namespaces, form]);

  return (
    <>
      <Dialog
        open={open}
        onOpenChange={(open) => {
          if (!open) {
            close();
          }
        }}
      >
        <DialogContent className="rounded-lg sm:rounded-lg">
          <form
            onSubmit={(e) => {
              e.preventDefault();
              e.stopPropagation();
              void form.handleSubmit();
            }}
            className="flex flex-col gap-4"
          >
            <DialogHeader>
              <DialogTitle>Create Queue</DialogTitle>
              <DialogDescription>
                There is no limit to the number of queues you can create.
              </DialogDescription>
            </DialogHeader>
            <form.Field name="name">
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor={field.name}>Name</Label>
                  <Input
                    id={field.name}
                    name={field.name}
                    value={field.state.value}
                    type="text"
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    placeholder="Name"
                    data-1p-ignore
                    className={cn(
                      "focus-visible:outline-none focus-visible:ring-0 focus-visible:ring-offset-0",
                      "focus:border-primary focus:border transition-all",
                    )}
                  />
                  {field.state.meta.errors ? (
                    <span className="text-sm text-destructive">
                      {field.state.meta.errors.join(", ")}
                    </span>
                  ) : null}
                </div>
              )}
            </form.Field>
            <form.Field name="namespace">
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor={field.name}>Namespace</Label>
                  <Popover open={nsPopoverOpen} onOpenChange={setNsPopoverOpen}>
                    <PopoverTrigger asChild>
                      <Button
                        variant="outline"
                        // biome-ignore lint/a11y/useSemanticElements: <explanation>
                        role="combobox"
                        className={cn(
                          "w-full justify-between",
                          field.state.value ? "" : "text-muted-foreground",
                        )}
                      >
                        {field.state.value
                          ? field.state.value
                          : "Select namespace"}
                        <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                      </Button>
                    </PopoverTrigger>
                    <PopoverContent className="w-[var(--radix-popover-trigger-width)] p-0">
                      <Command className="bg-background">
                        <CommandInput placeholder="Search namespace..." />
                        <CommandList>
                          <CommandEmpty>
                            {isLoading ? (
                              <Spinner />
                            ) : (
                              <div className="flex flex-col items-center justify-center py-4 gap-2">
                                <p className="text-sm text-muted-foreground">
                                  No namespace found.
                                </p>
                              </div>
                            )}
                          </CommandEmpty>
                          <CommandGroup>
                            {namespaces.map((namespace) => (
                              <CommandItem
                                key={namespace.name}
                                value={namespace.name}
                                className="cursor-pointer"
                                onSelect={(currentValue) => {
                                  field.handleChange(currentValue);
                                  setNsPopoverOpen(false);
                                }}
                              >
                                {namespace.name}
                              </CommandItem>
                            ))}
                          </CommandGroup>
                          <CommandGroup>
                            <CommandItem
                              onSelect={() => setShowCreateNamespace(true)}
                              className="flex items-center gap-2 cursor-pointer"
                            >
                              <Plus className="h-4 w-4" />
                              Create Namespace
                            </CommandItem>
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

            <form.Field name="tags">
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor={field.name}>Tags</Label>
                  <KeyValueForm
                    value={field.state.value}
                    onChange={(value) => field.handleChange(value)}
                  />
                  {field.state.meta.errors ? (
                    <span className="text-sm text-destructive">
                      {field.state.meta.errors.join(", ")}
                    </span>
                  ) : null}
                </div>
              )}
            </form.Field>

            <DialogFooter>
              <form.Subscribe
                selector={(state) => [state.canSubmit, state.isSubmitting]}
              >
                {([canSubmit, isSubmitting]) => (
                  <div className="flex flex-col sm:flex-row gap-2">
                    <Button type="submit" disabled={!canSubmit}>
                      {isSubmitting ? (
                        <>
                          {
                            <Spinner
                              className="absolute self-center"
                              size="sm"
                              color="current"
                            />
                          }
                          <p className="text-transparent">Create</p>
                        </>
                      ) : (
                        "Create"
                      )}
                    </Button>

                    <DialogClose asChild>
                      <Button variant={"secondary"} disabled={isSubmitting}>
                        Cancel
                      </Button>
                    </DialogClose>
                  </div>
                )}
              </form.Subscribe>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <CreateNamespace
        open={showCreateNamespace}
        close={() => setShowCreateNamespace(false)}
        onSuccess={handleNamespaceCreated}
      />
    </>
  );
}
