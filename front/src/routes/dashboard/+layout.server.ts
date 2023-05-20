import { redirect } from '@sveltejs/kit';
import type {LayoutServerLoad} from "../../../.svelte-kit/types/src/routes/dashboard/$types";

export const load: LayoutServerLoad = async ({ fetch }) => {
    const session = await fetch("/api/auth/check", {
        method: "GET",
        credentials: "include",
    });
    if (session.status !== 200) {
        throw redirect(307, '/login');
    }
    return;
}

