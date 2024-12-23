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
import { Label } from "./ui/label";
import { cn } from "@/lib/utils";
import {
  listNamespaces,
  listUserAllowedNamespaces,
  updateUserAllowedNamespaces,
  updateUserRole,
} from "@/lib/actions/api";
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
import { useInvalidate } from "@/lib/hooks/use-invalidate";
import CreateNamespace from "./create-namespace";
import { useState } from "react";
import { modifyUserSchema } from "@/lib/schemas/modify-user";
import type { UserStatistics } from "./create-user";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { Role, useSession } from "@/lib/state/global";

export default function ModifyUser({
  open,
  close,
  onSuccess,
  user,
}: {
  open: boolean;
  close: () => void;
  onSuccess?: (userName: string) => void;
  user?: UserStatistics;
}) {
  const [showCreateNamespace, setShowCreateNamespace] = useState(false);
  const [nsPopoverOpen, setNsPopoverOpen] = useState(false);

  const session = useSession();

  const { data: namespaces = [], isLoading } = useQuery({
    queryFn: () => listNamespaces(),
    queryKey: ["namespaces"],
  });

  const { data: userNamespaces, isLoading: userNamespacesLoading } = useQuery({
    queryKey: [
      "users",
      "namespaces",
      "user-namespaces",
      { email: user?.email },
    ],
    queryFn: () =>
      listUserAllowedNamespaces({
        email: session?.email,
      }),
  });

  const invalidate = useInvalidate(["users", "user-namespaces"]);

  const form = useForm({
    defaultValues: {
      email: user?.email ?? "",
      password: "",
      namespaces: new Set(userNamespaces ?? []) as Set<string>,
      role: user?.role ?? Role.User,
    },
    validatorAdapter: yupValidator(),
    validators: {
      onChange: modifyUserSchema,
      onMount: modifyUserSchema,
      onSubmit: modifyUserSchema,
    },
    onSubmit: async ({ value: data, formApi }) => {
      await Promise.all([
        updateUserAllowedNamespaces({
          email: session?.email ?? "",
          namespaces: Array.from(data.namespaces.keys()),
        }),
        updateUserRole({
          email: session?.email ?? "",
          role: data.role,
        }),
      ])
        .then(() => {
          invalidate();
          onSuccess?.(session?.email ?? "");
          close();
          formApi.reset();
        })
        .catch(() => {
          toast.error("Failed to update user");
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
              <DialogTitle>Modify User Access</DialogTitle>
              <DialogDescription>
                Modify namespace access for this user.
              </DialogDescription>
            </DialogHeader>
            <form.Field name="role">
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor={field.name}>Role</Label>
                  <Select
                    value={field.state.value}
                    onValueChange={(value) => field.handleChange(value as Role)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select a role" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="user">User</SelectItem>
                      <SelectItem value="admin">Admin</SelectItem>
                    </SelectContent>
                  </Select>
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
                  <Label htmlFor={field.name}>Grant Access to Namespaces</Label>
                  <Popover open={nsPopoverOpen} onOpenChange={setNsPopoverOpen}>
                    <PopoverTrigger asChild>
                      <Button
                        variant="outline"
                        className={cn(
                          "w-full justify-between",
                          field.state.value?.size > 0
                            ? ""
                            : "text-muted-foreground",
                        )}
                      >
                        {field.state.value?.size > 0
                          ? (Array.from(field.state.value).reduce(
                              (acc: string, curr: string, idx: number) => {
                                if (idx > 0) {
                                  return `${acc}, ${curr}`;
                                }
                                return curr;
                              },
                              "",
                            ) as string)
                          : "Select namespaces to grant access"}
                        <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                      </Button>
                    </PopoverTrigger>
                    <PopoverContent className="w-[var(--radix-popover-trigger-width)] p-0">
                      <Command className="bg-background">
                        <CommandInput placeholder="Search namespace..." />
                        <CommandList>
                          <CommandEmpty>
                            {isLoading || userNamespacesLoading ? (
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
                                  <div className="w-4 h-4 flex items-center justify-center">
                                    {field.state.value.has(namespace.name) && (
                                      <Check className="h-4 w-4" />
                                    )}
                                  </div>
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
                          <Spinner
                            className="absolute self-center"
                            size="sm"
                            color="current"
                          />
                          <p className="text-transparent">Save Changes</p>
                        </>
                      ) : (
                        "Save Changes"
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
