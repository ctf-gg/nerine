import { isError, profile } from "$lib/api";
import { jwtDecode } from "jwt-decode";

export const load = async ({ url }) => {
  const urlToken = url.searchParams.get("token");
  if (urlToken) {
    try {
      const { team_id } = jwtDecode<{ team_id: string }>(urlToken);

      if (team_id) {
        const prof = await profile(team_id, urlToken);

        if (isError(prof)) return { teamName: null, error: prof };

        return { teamName: prof.name, token: urlToken };
      }
    } catch (e: any) {
      return { teamName: null, error: { message: e.message } };
    }
  }
  return { teamName: undefined };
};
