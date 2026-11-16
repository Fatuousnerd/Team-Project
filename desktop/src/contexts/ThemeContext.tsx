import {
	createContext,
	type ReactNode,
	useCallback,
	useEffect,
	useState,
} from "react";
import type { ThemeMode, ThemeType } from "@/config/Types/app.types";

const STORAGE_KEY = "meridian.theme";

export const ThemeContext = createContext<ThemeType | null>(null);

function readInitialTheme(): ThemeMode {
	if (typeof window === "undefined") return "dark";
	const stored = window.localStorage.getItem(STORAGE_KEY);
	if (stored === "light" || stored === "dark") return stored;
	return "dark";
}

export const ThemeProvider = ({ children }: { children: ReactNode }) => {
	const [theme, setTheme] = useState<ThemeMode>(readInitialTheme);

	useEffect(() => {
		const root = document.documentElement;
		root.classList.toggle("dark", theme === "dark");
		root.style.colorScheme = theme;
		window.localStorage.setItem(STORAGE_KEY, theme);
	}, [theme]);

	const toggleTheme = useCallback(() => {
		setTheme((current) => (current === "dark" ? "light" : "dark"));
	}, []);

	return (
		<ThemeContext.Provider value={{ theme, setTheme, toggleTheme }}>
			{children}
		</ThemeContext.Provider>
	);
};
