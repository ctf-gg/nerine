import { isError, profile, getEvent } from "$lib/api";
import { jwtDecode } from "jwt-decode";

export const load = async ({ cookies }) => {
  const event = await getEvent();
  const token = cookies.get("token");
  if (!token) return { authedProfile: null, teamId: null, event };

  const teamId = jwtDecode<{ team_id: string }>(token).team_id;
  let prof = token ? await profile(teamId, token) : null;

  if (prof != null && isError(prof)) prof = null;

  return {
    teamId,
    authedProfile: prof,
    event,
  };
};
