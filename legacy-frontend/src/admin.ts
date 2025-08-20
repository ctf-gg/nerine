import { req, type ApiError, type RequestOptions } from "./api";

interface Category {
    id: number,
    name: string,
}


interface ChallengeGroup {
  id: number;
  name: string;
}

export interface AdminChallenge {
  id: number;
  public_id: string;
  name: string;
  author: string;
  description: string;
  points_min: number;
  points_max: number;
  flag: string;
  attachments: { [name: string]: string };
  visible: boolean;

  category: Category;
  group: ChallengeGroup | null;
}

const tokenToOptions = (
  token?: string
): Omit<RequestOptions, "body"> | undefined =>
  token ? { headers: { Cookie: `admin_token=${token}` } } : undefined;


export const adminChallenges = async (
  adminToken?: string
): Promise<AdminChallenge[] | ApiError> => {
  const res = await req("GET", "/admin/challs", tokenToOptions(adminToken));

  return (await res.json()) as AdminChallenge[] | ApiError;
};
