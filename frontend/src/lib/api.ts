export function getApiBase(): string {
  if (globalThis.window) {
    return "/api";
  } else {
    return import.meta.env.API_BASE ?? "http://nerine.localhost/api";
  }
}

export type ApiError = GenericApiError | EventNotStartedApiError;

export interface GenericApiError {
  error:
    | "database_error"
    | "jwt_error"
    | "validation_error"
    | "deploy_error"
    | "invalid_token"
    | "not_found"
    | "event_ended"
    | "wrong_flag"
    | "team_name_taken"
    | "generic_error";
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

export interface RequestOptions {
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

export const req: Req = (method, path, options = {}) => {
  if (method == "GET") {
    const { headers } = options;
    return fetch(`${getApiBase()}${path}`, { method, headers });
  } else {
    const { body, headers } = options as RequestOptions;
    return fetch(`${getApiBase()}${path}`, {
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

export const verifyEmail = async (
  token: string
): Promise<TeamId | ApiError> => {
  const res = await req("POST", "/auth/verify_email", {
    body: { token },
  });
  return (await res.json()) as TeamId | ApiError;
};

export interface VerificationDetailsTeamRegistration {
  verification_type: "team_registration";
  name: string;
  email: string;
}

export interface VerificationDetailsEmailUpdate {
  verification_type: "email_update";
  name: string;
  new_email: string;
}

export type VerificationDetails =
  | VerificationDetailsTeamRegistration
  | VerificationDetailsEmailUpdate;

export const getVerificationDetails = async (
  token: string
): Promise<VerificationDetails> => {
  const res = await req("POST", "/auth/verification_details", {
    body: { token },
  });
  return (await res.json()) as VerificationDetails;
};

export const updateProfile = async (
  email: string,
  name: string
): Promise<Team | ApiError | { message: string; name: string }> => {
  const res = await req("POST", "/profile/update", {
    body: { email, name },
  });
  const result = await res.json();
  return result as Team | ApiError | { message: string; name: string };
};

interface Solve {
  name: string;
  points: number;
  solved_at: string;
}

export type Profile =
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
  attachments: { [name: string]: string };
  strategy: "static" | "instanced";
  deployment_id: string | null;
  category: string;
  solved_at: Date | null;
}

export const challenges = async (
  token?: string
): Promise<Challenge[] | ApiError> => {
  const res = await req("GET", "/challs", tokenToOptions(token));
  const challs = (await res.json()) as Challenge[] | ApiError;
  if (isError(challs)) return challs;

  return challs.map((c) => ({
    ...c,
    solved_at: c.solved_at && new Date(c.solved_at + "Z"),
  }));
};

export interface Badge {
  type: string;
  obtained: string;
  chall: string;
}

export interface ScorePoint {
  date: string;
  score: number;
}

export interface LeaderboardEntry {
  id: string;
  name: string;
  score: number;
  score_history: ScorePoint[];
  extra: { badges: Badge[] };
}
export const leaderboard = async (): Promise<LeaderboardEntry[] | ApiError> => {
  const res = await req("GET", "/leaderboard");
  return (await res.json()) as LeaderboardEntry[] | ApiError;
};

export async function submitFlag(
  challengeId: string,
  flag: string,
  token?: string
): Promise<{} | ApiError> {
  const res = await req("POST", "/challs/submit", {
    ...tokenToOptions(token),
    body: {
      challenge_id: challengeId,
      flag,
    },
  });
  if (res.status == 200) {
    return {};
  } else {
    return await res.json();
  }
}

export interface Event {
  name: string;
  start_time: Date;
  end_time: Date;
}

// TODO(ani): don't assume this always succeeds
export async function getEvent(): Promise<Event> {
  const res = await req("GET", "/event");
  const j = await res.json();
  return {
    name: j.name,
    start_time: new Date(j.start_time),
    end_time: new Date(j.end_time),
  };
}

export const verifyEmailUpdate = async (token: string): Promise<Team> => {
  const res = await req("POST", "/profile/verify_email_update", {
    body: { token },
  });
  return (await res.json()) as Team;
};

export async function resendToken(email: string) {
  await req("POST", "/auth/resend_token", { body: { email } });
}

export interface ChallengeDeployment {
  id: string;
  deployed: boolean;
  data: { [k: string]: DeploymentData };
  created_at: string; // TODO make into date
  expired_at: string | null;
  destroyed_at: string | null;
}

interface DeploymentData {
  ports: { [port: number]: HostMapping };
}

type HostMapping =
  | { type: "tcp"; port: number }
  | { type: "http"; subdomain: string; base: string };

export async function deployChallenge(
  challengeId: string
): Promise<ChallengeDeployment | ApiError> {
  // TODO make it so we don't have to include body in post
  const res = await req("POST", "/challs/deploy/new/" + challengeId, {
    body: {},
  });

  return (await res.json()) as ChallengeDeployment | ApiError;
}

export async function destroyChallenge(
  challengeId: string
): Promise<"ok" | ApiError> {
  // TODO make it so we don't have to include body in post
  const res = await req("DELETE", "/challs/deploy/destroy/" + challengeId, {
    body: {},
  });

  return (await res.json()) as "ok" | ApiError;
}

export async function getChallengeDeployment(
  deploymentId: string
): Promise<ChallengeDeployment | ApiError> {
  const res = await req("GET", "/challs/deploy/get/" + deploymentId);

  return (await res.json()) as ChallengeDeployment | ApiError;
}
