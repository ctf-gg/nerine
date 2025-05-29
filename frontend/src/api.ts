export const API_BASE = "http://sctf.localhost/api";

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

interface RequestOptions {
  body: object;
  headers?: HeadersInit;
}

type Req = {
  (
    method: "GET",
    path: string,
    options?: Omit<RequestOptions, "body">
  ): Promise<Response>;
  (
    method: "POST" | "PUT" | "DELETE",
    path: string,
    options: RequestOptions
  ): Promise<Response>;
};

const req: Req = (method, path, options = {}) => {
  if (method == "GET") {
    const { headers } = options;
    return fetch(`${API_BASE}${path}`, { method, headers });
  } else {
    const { body, headers } = options as RequestOptions;
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

interface TeamId {
  id: string;
}

export const login = async (token: string): Promise<TeamId | ApiError> => {
  const res = await req("POST", "/auth/login", {
    body: { token },
  });

  return (await res.json()) as TeamId | ApiError;
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

const tokenToOptions = (
  token?: string
): Omit<RequestOptions, "body"> | undefined =>
  token ? { headers: { Cookie: `token=${token}` } } : undefined;

export const profile = async (
  id: string,
  token?: string
): Promise<Profile | ApiError> => {
  const res = await req("GET", `/profile/${id}`, tokenToOptions(token));

  return (await res.json()) as Profile | ApiError;
};

interface Token {
  token: string;
}

export const genToken = async (): Promise<Token | ApiError> => {
  const res = await req("GET", "/auth/gen_token");

  return (await res.json()) as Token | ApiError;
};

export interface Challenge {
  id: string;
  name: string;
  author: string;
  description: string;
  points: number;
  solves: number;
  attachments: any;
  category: string;
}

export const challenges = async (
  token?: string
): Promise<Challenge[] | ApiError> => {
  const res = await req("GET", "/challs", tokenToOptions(token));
  return (await res.json()) as Challenge[] | ApiError;
};

interface LeaderboardEntry {
  id: string;
  name: string;
  score: number;
}

export const leaderboard = async (
  token?: string
): Promise<LeaderboardEntry[] | ApiError> => {
  const res = await req("GET", "/leaderboard", tokenToOptions(token));
  return (await res.json()) as LeaderboardEntry[] | ApiError;
};
