import { redirect } from '@sveltejs/kit';

export const actions = {
  default: async ({ cookies }) => {
    cookies.delete('token', { path: '/' });

    redirect(303, '/');
  }
};
