import { isError, challenges, leaderboard } from "$lib/api.js";
import { event } from "$lib/event.js";
import { redirect } from "@sveltejs/kit";

export const load = async ({ cookies }) => {
  const eventStarted = new Date().getTime() >= event.start_time.getTime();
  if (!eventStarted) return { leaderboard: null };

  const lb = await leaderboard();

  if (isError(lb)) {
    if (lb.error === "invalid_token") redirect(307, "/login");
    else throw new Error("fetching challenges failed");
  }

  return { leaderboard: lb };
};
