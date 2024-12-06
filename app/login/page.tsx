"use client";

import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardHeader,
  CardContent,
  CardFooter,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";
import { useForm } from "@tanstack/react-form";
import { type YupValidator, yupValidator } from "@tanstack/yup-form-adapter";
import { loginFormSchema, type LoginRequest } from "@/schemas/login-form";
import { type AdminSession, useGlobalState } from "@/lib/state/global";
import { login } from "@/actions/api";

// Login page component with form validation using TanStack Form and Yup
export default function LoginPage() {
  const router = useRouter();

  // Initialize form with validation schema and submission handler
  const form = useForm<LoginRequest, YupValidator>({
    validatorAdapter: yupValidator(),
    validators: {
      onSubmit: loginFormSchema,
      onChange: loginFormSchema,
      onMount: loginFormSchema,
    },
    defaultValues: {
      email: "",
      password: "",
    },
    onSubmit: async ({ value }) => {
      "use client";

      const data: AdminSession | undefined = await login(value).catch(
        (e: Error) => {
          toast.error(e.message);
          return undefined;
        },
      );

      if (data === undefined) {
        return;
      }

      useGlobalState.setState({ session: data });

      router.replace("/queues");
    },
  });

  return (
    // Centered login card with responsive layout
    <div className="min-h-screen w-full flex items-center justify-center">
      <Card className="w-96">
        <CardHeader>
          <h1 className="text-2xl font-bold">Login</h1>
        </CardHeader>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            e.stopPropagation();
            form.handleSubmit();
          }}
        >
          <CardContent className="flex flex-col gap-4">
            <form.Field name="email">
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor={field.name}>Email</Label>
                  <Input
                    type="text"
                    name={field.name}
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    className="w-full p-2 border rounded"
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
                  <Label htmlFor="password">Password</Label>
                  <Input
                    type="password"
                    name={field.name}
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    className="w-full p-2 border rounded"
                  />
                  {field.state.meta.errors ? (
                    <span className="text-sm text-destructive">
                      {field.state.meta.errors.join(", ")}
                    </span>
                  ) : null}
                </div>
              )}
            </form.Field>
          </CardContent>
          <CardFooter>
            <Button type="submit" className="w-full">
              Sign In
            </Button>
          </CardFooter>
        </form>
      </Card>
    </div>
  );
}
