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
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { cn } from "@/lib/utils";
import { createNamespace } from "@/actions/api";
import { Spinner } from "@nextui-org/react";
import { toast } from "sonner";
import { useInvalidate } from "@/hooks/use-invalidate";
import { createNamespaceSchema } from "@/schemas/create-namespace";
import { listNamespaces } from "@/actions/api";
import { useQuery } from "@tanstack/react-query";

interface CreateNamespaceProps {
  open: boolean;
  close: () => void;
  onSuccess?: (namespaceName: string) => void;
}

export default function CreateNamespace({
  open,
  close,
  onSuccess,
}: CreateNamespaceProps) {
  const invalidate = useInvalidate(["namespaces"]);

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { data: namespaces = [], isLoading } = useQuery({
    queryFn: () => listNamespaces(),
    queryKey: ["namespaces"],
  });

  const form = useForm({
    defaultValues: {
      name: "",
      role: "user",
    },
    validatorAdapter: yupValidator(),
    validators: {
      onChange: createNamespaceSchema,
      onMount: createNamespaceSchema,
    },
    onSubmit: async ({ value: data, formApi }) => {
      if (namespaces.some(namespace => namespace.name === data.name)) {
        toast.error("This name is already taken");
        return;
      }

      await createNamespace(data)
        .then(() => {
          invalidate();
          if (onSuccess) {
            onSuccess(data.name);
          }
          close();
          formApi.reset();
        })
        .catch(() => {
          toast.error("Something went wrong");
        });
    },
  });

  return (
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
            <DialogTitle>Create Namespace</DialogTitle>
            <DialogDescription>
              Create a new namespace to organize your queues.
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

          <DialogFooter>
            <form.Subscribe
              selector={(state) => [state.canSubmit, state.isSubmitting]}
            >
              {([canSubmit, isSubmitting]) => (
                <div className="flex flex-col sm:flex-row gap-2">
                  <Button type="submit" disabled={!canSubmit}>
                    {isSubmitting ? (
                      <>
                        <Spinner
                          className="absolute self-center"
                          size="sm"
                          color="current"
                        />
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
  );
}
