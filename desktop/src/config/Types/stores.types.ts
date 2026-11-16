import type { ErrorType } from "./app.types";

export type LoadingStore = {
	loading: Record<string, boolean>;
	setLoading: (key: string, value: boolean) => void;
};

export type ErrStore = {
	errs: Record<string, ErrorType>;
	setErr: (key: string, value: ErrorType) => void;
};
