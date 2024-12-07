import { Role } from "@/lib/state/global";
import { type InferType, array, object, string } from "yup";

export const modifyUserSchema = object({
  namespaces: array()
    .of(string().required())
    .optional()
    .transform((v: Set<string>) => [...v.values()]),
  role: string().required().oneOf<Role>([Role.Admin, Role.User]),
});

export type ModifyUserRequest = InferType<typeof modifyUserSchema>;
