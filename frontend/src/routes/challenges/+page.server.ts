import { isError, challenges } from "$lib/api.js";
import { redirect } from "@sveltejs/kit";

export const load = async ({ cookies, parent }) => {
  const { event } = await parent();
  const token = cookies.get("token");
  const eventStarted = new Date().getTime() >= event.start_time.getTime();

  const challs = await challenges(token);

  if (isError(challs)) {
    if (challs.error === "invalid_token") {
      if (!eventStarted) {
        return { challs: null };
      } else {
        redirect(307, "/login");
      }
    }
    if (challs.error === "event_not_started") return { challs: null };
    else throw new Error("fetching challenges failed");
  }

  return { challs };
};
