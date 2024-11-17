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
import { yupValidator } from "@tanstack/yup-form-adapter";
import { useQuery } from "@tanstack/react-query";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { cn } from "@/lib/utils";
import { createUser, listNamespaces } from "@/actions/api";
import { Spinner } from "@nextui-org/react";
import { ChevronsUpDown, Plus, Check } from "lucide-react";
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
import { useInvalidate } from "@/hooks/use-invalidate";
import CreateNamespace from "./create-namespace";
import { useState } from "react";
import { createUserSchema } from "@/schemas/create-user";

export interface UserStatistics {
  email: string;
  // role: string;
  // createdAt: string;
  // lastLogin?: string;
  // namespaces: string[];
}

export default function CreateUser({
  open,
  close,
  onSuccess,
}: {
  open: boolean;
  close: () => void;
  onSuccess?: (userName: string) => void;
}) {
  const [showCreateNamespace, setShowCreateNamespace] = useState(false);
  const [nsPopoverOpen, setNsPopoverOpen] = useState(false);

  const { data: namespaces = [], isLoading } = useQuery({
    queryFn: () => listNamespaces(),
    queryKey: ["namespaces"],
  });

  const invalidate = useInvalidate(["users"]);

  const form = useForm({
    defaultValues: {
      email: "",
      password: "",
      namespaces: new Set() as Set<string>,
    },
    validatorAdapter: yupValidator(),
    validators: {
      onChange: createUserSchema,
      onMount: createUserSchema,
      onSubmit: createUserSchema,
    },
    onSubmit: async ({ value: data }) => {
      await createUser({
        email: data.email,
        password: data.password,
        namespaces: [...data.namespaces.keys()],
      })
        .then(() => {
          invalidate();
          onSuccess?.(data.email);
        })
        .catch(() => {
          toast.error("Something went wrong");
        })
        .finally(() => {
          close();
        });
    },
  });

  const handleNamespaceCreated = async (namespaceName: string) => {
    form.setFieldValue("namespaces", (set) => {
      set.add(namespaceName);
      return set;
    });
    await form.validateField("namespaces", "change");
    setShowCreateNamespace(false);
  };

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
        <DialogContent>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              e.stopPropagation();
              void form.handleSubmit();
            }}
            className="flex flex-col gap-4"
          >
            <DialogHeader>
              <DialogTitle>Create User</DialogTitle>
              <DialogDescription>
                Create a new user with access to selected namespaces.
              </DialogDescription>
            </DialogHeader>
            <form.Field name="email">
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor={field.name}>Name</Label>
                  <Input
                    id={field.name}
                    name={field.name}
                    type="text"
                    value={field.state.value}
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
            <form.Field name="password">
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor={field.name}>Name</Label>
                  <Input
                    id={field.name}
                    name={field.name}
                    type="password"
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    placeholder="Password"
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
            <form.Field
              defaultValue={new Set() as Set<string>}
              name="namespaces"
            >
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor={field.name}>Namespaces</Label>
                  <Popover open={nsPopoverOpen} onOpenChange={setNsPopoverOpen}>
                    <PopoverTrigger asChild>
                      <Button
                        variant="outline"
                        // biome-ignore lint/a11y/useSemanticElements: <explanation>
                        role="combobox"
                        className={cn(
                          "w-full justify-between",
                          field.state.value?.size > 0
                            ? ""
                            : "text-muted-foreground",
                        )}
                      >
                        {field.state.value?.size > 0
                          ? (field.state.value
                              .values()
                              .reduce((acc, curr, idx) => {
                                if (idx > 0) {
                                  return `${acc}, ${curr}`;
                                }
                                return curr;
                              }, "") as string)
                          : "Select namespaces"}
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
                                  const currentNamespaces = field.state.value;
                                  if (!currentNamespaces.has(currentValue)) {
                                    currentNamespaces.add(currentValue);
                                  } else {
                                    currentNamespaces.delete(currentValue);
                                  }
                                  field.handleChange(currentNamespaces);
                                }}
                              >
                                <div className="flex items-center gap-2">
                                  {field.state.value.has(namespace.name) ? (
                                    <Check className="h-4 w-4" />
                                  ) : null}
                                  {namespace.name}
                                </div>
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

            <DialogFooter>
              <form.Subscribe
                selector={(state) => [state.canSubmit, state.isSubmitting]}
              >
                {([canSubmit, isSubmitting]) => (
                  <>
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
                  </>
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
