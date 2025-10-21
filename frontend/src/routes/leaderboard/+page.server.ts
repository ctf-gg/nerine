import { isError, challenges, leaderboard } from "$lib/api";
import { redirect } from "@sveltejs/kit";

export const load = async ({ cookies }) => {
  const { event } = await parent();
  const eventStarted = new Date().getTime() >= event.start_time.getTime();
  if (!eventStarted) return { leaderboard: null, event };

  const lb = await leaderboard();

  if (isError(lb)) {
    if (lb.error === "invalid_token") redirect(307, "/login");
    else throw new Error("fetching challenges failed");
  }

  return { leaderboard: lb, event };
};
