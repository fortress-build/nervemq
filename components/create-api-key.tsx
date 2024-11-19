import { useState, useCallback } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogTitle,
  DialogClose,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { cn, isAlphaNumeric } from "@/lib/utils";
import { type InferType, object, string } from "yup";
import { useForm } from "@tanstack/react-form";
import { yupValidator } from "@tanstack/yup-form-adapter";
import { Spinner } from "@nextui-org/react";
import { Input } from "@/components/ui/input";
import { useInvalidate } from "@/hooks/use-invalidate";
import { DialogHeader } from "./ui/dialog";
import { createAPIKey } from "@/actions/api";

// Add schema
export const createApiKeySchema = object({
  name: string()
    .required()
    .max(32)
    .min(1)
    .test("name", "name should be alphanumeric", (value: string) => {
      return isAlphaNumeric(value);
    }),
});

export type CreateApiKey = InferType<typeof createApiKeySchema>;

export interface APIKey {
  // id: string;
  name: string;
  // created_at: string;
  // last_used?: string;
  key?: string;
}

interface CreateApiKeyProps {
  open: boolean;
  close: () => void;
  onSuccess?: (keyName: string) => void;
}

export default function CreateApiKey({
  open,
  close,
  onSuccess,
}: CreateApiKeyProps) {
  const [showKey, setShowKey] = useState(false);
  const [apiKey, setApiKey] = useState<APIKey | null>(null);
  const invalidate = useInvalidate(["apiKeys"]);

  const copyToClipboard = useCallback(() => {
    if (apiKey?.key) {
      navigator.clipboard.writeText(apiKey.key);
      toast.success("API key copied to clipboard");
    }
  }, [apiKey]);

  const downloadKey = useCallback(() => {
    if (apiKey?.key) {
      const blob = new Blob([apiKey.key], { type: "text/plain" });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `api-key-${apiKey.name}.txt`;
      a.click();
      window.URL.revokeObjectURL(url);
    }
  }, [apiKey]);

  const form = useForm({
    defaultValues: {
      name: "",
    },
    validatorAdapter: yupValidator(),
    validators: {
      onChange: createApiKeySchema,
      onMount: createApiKeySchema,
    },
    onSubmit: async ({ value: data, formApi }) => {
      await createAPIKey(data)
        .then((result) => {
          setApiKey(result);
          setShowKey(true);
          invalidate();
          if (onSuccess) {
            onSuccess(data.name);
          }
          formApi.reset();
        })
        .catch(() => {
          toast.error("Failed to create API key");
        });
    },
  });

  return (
    <Dialog
      open={open}
      onOpenChange={(open) => {
        if (!open) {
          close();
          setShowKey(false);
          setApiKey(null);
        }
      }}
    >
      <DialogContent>
        {!showKey ? (
          <form
            onSubmit={(e) => {
              e.preventDefault();
              e.stopPropagation();
              void form.handleSubmit();
            }}
            className="flex flex-col gap-4"
          >
            <DialogHeader>
              <DialogTitle>Create API Key</DialogTitle>
              <DialogDescription>
                Create a new API key for accessing the API.
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
                    placeholder="My API Key"
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
                      <Button variant="secondary" disabled={isSubmitting}>
                        Cancel
                      </Button>
                    </DialogClose>
                  </>
                )}
              </form.Subscribe>
            </DialogFooter>
          </form>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle>API Key Created</DialogTitle>
              <DialogDescription>
                Please copy or download your API key now. You won&apos;t be able
                to see it again!
              </DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="flex items-center gap-2">
                <Input
                  readOnly
                  value={apiKey?.key}
                  type="text"
                  className="font-mono"
                />
              </div>
              <div className="grid gap-2">
                <Button onClick={copyToClipboard}>Copy to Clipboard</Button>
                <Button onClick={downloadKey}>Download API Key</Button>
              </div>
            </div>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
