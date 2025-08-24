import { isError, profile } from "$lib/api.js";
import { error, redirect } from "@sveltejs/kit";

export const load = async ({ cookies, params }) => {
  const token = cookies.get("token");

  let prof = await profile(params.team, token);

  if (isError(prof)) {
    if (prof.error === "invalid_token") redirect(307, "/login");
    else error(404, "profile not found");
  }

  return { profile: prof };
};
