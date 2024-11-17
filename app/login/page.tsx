"use client";

import { signIn } from "next-auth/react";
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
import { yupValidator } from "@tanstack/yup-form-adapter";
import { loginFormSchema } from "@/schemas/login-form";

export default function LoginPage() {
  const router = useRouter();

  const form = useForm({
    validatorAdapter: yupValidator(),
    validators: {
      onChange: loginFormSchema,
      onMount: loginFormSchema,
    },
    defaultValues: {
      email: "",
      password: "",
    },
    onSubmit: async ({ value }) => {
      const response = await signIn("credentials", {
        username: value.email,
        password: value.password,
        redirect: false,
      });

      if (response === undefined) {
        toast.error("Something went wrong");
        return;
      }

      if (!response.ok) {
        toast.error(response.error ?? "Something went wrong");
        return;
      }

      router.replace("/queues");
    },
  });

  return (
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
                    type="email"
                    name={field.name}
                    className="w-full p-2 border rounded"
                  />
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
                    className="w-full p-2 border rounded"
                  />
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
