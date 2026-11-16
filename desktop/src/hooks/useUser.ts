import type { User } from "@supabase/supabase-js";
import { useEffect, useState } from "react";
import { supabase } from "@/lib/supabase/supabase";
import { useLoading } from "@/lib/zustand/app.store";

export function useUser() {
	const [user, setUser] = useState<User | null>(null);
	const loading = useLoading((s) => s.loading.user);
	const setLoading = useLoading((s) => s.setLoading);

	useEffect(() => {
		supabase.auth
			.getSession()
			.then(({ data: { session } }) => {
				setUser(session?.user ?? null);
				setLoading("user", false);
			})
			.catch((err) => {
				console.error("Error: ", err);
				setLoading("user", false);
			});

		const {
			data: { subscription },
		} = supabase.auth.onAuthStateChange((_event, session) => {
			setUser(session?.user ?? null);
			setLoading("user", false);
		});

		return () => subscription.unsubscribe();
	}, [setLoading]);

	return { user, loading };
}
