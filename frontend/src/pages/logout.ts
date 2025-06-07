import { type APIRoute } from "astro";

export const prerender = false;
export const GET: APIRoute = ({ request }) => {
  return new Response(null, {
    status: 303,
    headers: {
      location: request.headers.get("referer") ?? "/",
      "set-cookie": "token=deleted; path=/; max-age=-1",
    }
  });
};
