import { isError, challenges, leaderboard } from "$lib/api";
import { redirect, error } from "@sveltejs/kit";

export const load = async ({ cookies, parent, url }) => {
  const { event } = await parent();
  const eventStarted = new Date().getTime() >= event.start_time.getTime();
  if (!eventStarted) return { leaderboard: null, event };

  const division = url.searchParams.get("division") ?? null;
  const lb = await leaderboard(division);

  if (isError(lb)) {
    if (lb.error === "invalid_token") redirect(307, "/login");
    else if (lb.error == "not_found") error(404, { message: "Division Not Found" });
    else throw new Error("fetching challenges failed");
  }

  return { leaderboard: lb, event, division };
};
