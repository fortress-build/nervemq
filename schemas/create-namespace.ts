import { isAlphaNumeric } from "@/lib/utils";
import { type InferType, object, string } from "yup";

export const createNamespaceSchema = object({
  name: string()
    .required()
    .max(32)
    .min(1)
    .test("name", "name should be alphanumeric", (value: string) => {
      return isAlphaNumeric(value);
    }),
  role: string()
    .required()
    .oneOf(["admin", "user"], "Role must be either 'admin' or 'user'"),
});

export type CreateNamespaceRequest = InferType<typeof createNamespaceSchema>;
