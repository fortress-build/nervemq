import { type InferType, array, object, string } from "yup";

export const createUserSchema = object({
  email: string().required().email(),
  password: string().required().min(8).max(32),
  namespaces: array()
    .of(string().required())
    .required()
    .min(1)
    .transform((v: Set<string>) => [...v.values()]),
});

export type CreateUserRequest = InferType<typeof createUserSchema>;
