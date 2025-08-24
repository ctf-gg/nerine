export const load = async ({ cookies }) => {
  cookies.delete("token", { path: "/" });
};
