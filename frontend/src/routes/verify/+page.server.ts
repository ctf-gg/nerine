import { redirect } from "@sveltejs/kit";
import { getVerificationDetails, isError } from "$lib/api"; // Adjust path as needed

export const load = async ({ url }) => {
  const token = url.searchParams.get("token");
  if (!token) redirect(307, "/register");

  const detailsRes = await getVerificationDetails(token);

  if (isError(detailsRes)) redirect(307, "/register");
  return {
    verification: detailsRes,
    verificationToken: token,
  };
};
