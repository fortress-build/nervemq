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

import { type InferType, object, string } from "yup";
import { useForm } from "@tanstack/react-form";
import { yupValidator } from "@tanstack/yup-form-adapter";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { cn } from "@/lib/utils";
import { createNamespace } from "@/actions/api";
import { Spinner } from "@nextui-org/react";
import { toast } from "sonner";

function isAlphaNumeric(str: string) {
  let code: number;
  let i: number;
  let len: number;

  for (i = 0, len = str.length; i < len; i++) {
    code = str.charCodeAt(i);
    if (
      !(code > 47 && code < 58) && // numeric (0-9)
      !(code > 64 && code < 91) && // upper alpha (A-Z)
      !(code > 96 && code < 123)
    ) {
      // lower alpha (a-z)
      return false;
    }
  }
  return true;
}

const createNamespaceSchema = object({
  name: string()
    .required()
    .max(32)
    .min(1)
    .test("name", "name should be alphanumeric", (value: string) => {
      return isAlphaNumeric(value);
    }),
});

export type CreateNamespace = InferType<typeof createNamespaceSchema>;

export default function CreateNamespace({
  open,
  close,
}: {
  open: boolean;
  close: () => void;
}) {
  const form = useForm({
    defaultValues: {
      name: "",
    },
    validatorAdapter: yupValidator(),
    validators: {
      onChange: createNamespaceSchema,
      onMount: createNamespaceSchema,
    },
    onSubmit: async ({ value: data }) => {
      await createNamespace(data.name).catch((e) => {
        toast.error(e.message);
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
  );
}
