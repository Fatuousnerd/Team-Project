import { create } from "zustand";
import type { ErrStore, LoadingStore } from "@/config/Types/stores.types";

export const useLoading = create<LoadingStore>((set) => ({
	loading: { user: true },
	setLoading: (key, value) =>
		set((state) => ({ loading: { ...state.loading, [key]: value } })),
}));

export const useErrs = create<ErrStore>((set) => ({
	errs: {},
	setErr: (key, value) =>
		set((state) => ({ errs: { ...state.errs, [key]: value } })),
}));
