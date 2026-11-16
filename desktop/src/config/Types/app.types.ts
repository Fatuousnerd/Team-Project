export type ThemeMode = "light" | "dark";

export type ThemeType = {
	theme: ThemeMode;
	setTheme: (theme: ThemeMode) => void;
	toggleTheme: () => void;
};

export type ErrorType = TypeError | Error;
