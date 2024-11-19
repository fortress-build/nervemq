import { type InferType, array, object, string } from "yup";

export const modifyUserSchema = object({
  namespaces: array()
    .of(string().required())
    .optional()
    .transform((v: Set<string>) => [...v.values()]),
  role: string().required().oneOf(['admin', 'user']),
});

export type ModifyUserRequest = InferType<typeof modifyUserSchema>;
