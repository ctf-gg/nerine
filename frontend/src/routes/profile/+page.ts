import { redirect } from "@sveltejs/kit";

export const load = async ({ parent }) => {
  const data = await parent();
  redirect(307, `/profile/${data.teamId}`);
};
