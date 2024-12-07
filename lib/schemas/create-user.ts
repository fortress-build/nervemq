import { type InferType, array, object, string } from "yup";

export const createUserSchema = object({
  email: string().required().email(),
  password: string().required().min(8).max(32),
  namespaces: array()
    .of(string().required())
    .optional()
    .transform((v: Set<string>) => [...v.values()]),
  role: string().required().oneOf(['admin', 'user']),
});

export type CreateUserRequest = InferType<typeof createUserSchema>;
