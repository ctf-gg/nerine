import { isError, challenges } from "$lib/api.js";
import { event } from "$lib/event.js";
import { redirect } from "@sveltejs/kit";

export const load = async ({ cookies }) => {
  const token = cookies.get("token");
  const eventStarted = new Date().getTime() >= event.start_time.getTime();
  if (!eventStarted) return { challs: null };

  let challs = await challenges(token);

  if (isError(challs)) {
    if (challs.error === "invalid_token") redirect(307, "/login");
    else throw new Error("fetching challenges failed");
  }

  return { challs };
};
