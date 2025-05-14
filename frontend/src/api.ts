export const API_BASE = "http://localhost:3000/api";

export type ApiError = GenericApiError | EventNotStartedApiError;

export interface GenericApiError {
  error:
    | "database_error"
    | "jwt_error"
    | "invalid_token"
    | "not_found"
    | "event_ended"
    | "wrong_flag";
  message: string;
}

export interface EventNotStartedApiError {
  error: "event_not_started";
  message: string;
  data: string;
}

export interface Team {
  id: string;
  name: string;
  email: string;
  created_at: Date;
}

interface RequestOptions<T extends object> {
  body: T;
  headers?: { [k: string]: string | undefined };
}

type Req = {
  (
    method: "GET",
    path: string,
    options?: Omit<RequestOptions<object>, "body">
  ): Promise<Response>;
  (
    method: "POST" | "PUT" | "DELETE",
    path: string,
    options: RequestOptions<object>
  ): Promise<Response>;
};

const req: Req = (method, path, { body, headers } = {}) => {
  if (method == "GET") {
    return fetch(`${API_BASE}${path}`, { method, headers });
  } else {
    return fetch(`${API_BASE}${path}`, {
      method,
      body: JSON.stringify(body),
      headers: { "Content-Type": "application/json", ...headers },
    });
  }
};

export const isError = <T extends object>(
  response: T | ApiError
): response is ApiError => {
  return "error" in response && "message" in response;
};

export const register = async (
  email: string,
  name: string
): Promise<Team | ApiError> => {
  const res = await req("POST", "/auth/register", {
    body: { email, name },
  });

  return (await res.json()) as Team | ApiError;
};

interface Solve {
  name: string;
  points: number;
}

type Profile =
  | {
      type: "private";
      name: string;
      email: string;
      score: number;
      rank: number;
      solves: Solve[];
    }
  | {
      type: "public";

      name: string;
      score: number;
      rank: number;
      solves: Solve[];
    };

export const profile = async (id: string): Promise<Profile | ApiError> => {
  const res = await req("GET", `/profile/${id}`);

  return (await res.json()) as Profile | ApiError;
};
