import { goto } from "$app/navigation";

export const load = async () => {  
  goto("/", { invalidateAll: true });
};
